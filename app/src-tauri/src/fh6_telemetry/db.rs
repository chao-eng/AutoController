use rusqlite::{Connection, Result};
use std::path::PathBuf;

pub fn db_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("fh6-tel")
        .join("sessions.db")
}

pub fn open() -> Result<Connection> {
    let path = db_path();
    std::fs::create_dir_all(path.parent().unwrap()).ok();
    let conn = Connection::open(&path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
    init(&conn)?;
    Ok(conn)
}

pub fn init(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY,
            started_at INTEGER NOT NULL,
            ended_at INTEGER,
            car_ordinal INTEGER NOT NULL DEFAULT 0,
            car_class INTEGER NOT NULL DEFAULT 0,
            car_pi INTEGER NOT NULL DEFAULT 0,
            best_lap REAL,
            packet_count INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS session_packets (
            id INTEGER PRIMARY KEY,
            session_id INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
            timestamp_ms INTEGER NOT NULL,
            data BLOB NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_packets_session ON session_packets(session_id);",
    )?;
    migrate(conn);
    Ok(())
}

/// Additive, backwards-compatible migrations. Each ALTER is idempotent: on an
/// already-migrated DB SQLite returns a "duplicate column" error which we
/// intentionally ignore. Never drops or rewrites existing data.
fn migrate(conn: &Connection) {
    let _ = conn.execute("ALTER TABLE sessions ADD COLUMN name TEXT", []);
    let _ = conn.execute(
        "ALTER TABLE sessions ADD COLUMN bookmarked INTEGER NOT NULL DEFAULT 0",
        [],
    );
    // Per-lap times. UNIQUE(session_id, lap_number) so a re-driven lap (rewind)
    // overwrites rather than duplicating.
    let _ = conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS session_laps (
            id INTEGER PRIMARY KEY,
            session_id INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
            lap_number INTEGER NOT NULL,
            lap_time REAL NOT NULL,
            UNIQUE(session_id, lap_number)
        );
        CREATE INDEX IF NOT EXISTS idx_laps_session ON session_laps(session_id);",
    );
}

pub fn insert_lap(conn: &Connection, session_id: i64, lap_number: i64, lap_time: f32) -> Result<()> {
    conn.execute(
        "INSERT INTO session_laps (session_id, lap_number, lap_time) VALUES (?1, ?2, ?3)
         ON CONFLICT(session_id, lap_number) DO UPDATE SET lap_time=excluded.lap_time",
        rusqlite::params![session_id, lap_number, lap_time],
    )?;
    Ok(())
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LapRow {
    pub lap_number: i64,
    pub lap_time: f32,
}

/// Fastest recorded lap for a session, or None if it has no laps. Derived from
/// the lap table so it reflects rewind upserts/overwrites correctly.
pub fn min_lap_time(conn: &Connection, session_id: i64) -> Result<Option<f32>> {
    conn.query_row(
        "SELECT MIN(lap_time) FROM session_laps WHERE session_id=?1",
        [session_id],
        |r| r.get::<_, Option<f32>>(0),
    )
}

pub fn get_session_laps(conn: &Connection, session_id: i64) -> Result<Vec<LapRow>> {
    let mut stmt = conn.prepare(
        "SELECT lap_number, lap_time FROM session_laps
         WHERE session_id=?1 ORDER BY lap_number ASC",
    )?;
    let rows = stmt.query_map([session_id], |r| {
        Ok(LapRow { lap_number: r.get(0)?, lap_time: r.get(1)? })
    })?;
    rows.collect()
}

pub fn open_session(
    conn: &Connection,
    started_at: i64,
    car_ordinal: i32,
    car_class: i32,
    car_pi: i32,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO sessions (started_at, car_ordinal, car_class, car_pi) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![started_at, car_ordinal, car_class, car_pi],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Sets car metadata on a session that still has car_ordinal=0.
/// Called every packet so the first non-zero ordinal wins, handling the case
/// where the opening packet arrives before the game has populated car data.
pub fn update_session_car_if_unknown(
    conn: &Connection,
    id: i64,
    car_ordinal: i32,
    car_class: i32,
    car_pi: i32,
) -> Result<()> {
    conn.execute(
        "UPDATE sessions SET car_ordinal=?1, car_class=?2, car_pi=?3 WHERE id=?4 AND car_ordinal=0",
        rusqlite::params![car_ordinal, car_class, car_pi, id],
    )?;
    Ok(())
}

pub fn reopen_session(conn: &Connection, id: i64) -> Result<()> {
    conn.execute(
        "UPDATE sessions SET ended_at = NULL WHERE id=?1",
        [id],
    )?;
    Ok(())
}

pub fn close_session(conn: &Connection, id: i64, ended_at: i64, best_lap: f32) -> Result<()> {
    // best_lap is the authoritative MIN of the session's lap table (computed by
    // the caller), so the latest close is correct — including after a rewind
    // overwrote a provisional partial with the true lap. <=0 means "no laps":
    // keep whatever was there.
    conn.execute(
        "UPDATE sessions SET ended_at=?1,
         best_lap = CASE WHEN ?2 > 0.0 THEN ?2 ELSE best_lap END
         WHERE id=?3",
        rusqlite::params![ended_at, best_lap, id],
    )?;
    Ok(())
}

pub fn insert_packet(
    conn: &Connection,
    session_id: i64,
    timestamp_ms: u32,
    data: &[u8],
) -> Result<()> {
    conn.execute(
        "INSERT INTO session_packets (session_id, timestamp_ms, data) VALUES (?1, ?2, ?3)",
        rusqlite::params![session_id, timestamp_ms, data],
    )?;
    conn.execute(
        "UPDATE sessions SET packet_count = packet_count + 1 WHERE id=?1",
        [session_id],
    )?;
    Ok(())
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRow {
    pub id: i64,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub car_ordinal: i32,
    pub car_class: i32,
    pub car_pi: i32,
    pub best_lap: Option<f32>,
    pub packet_count: i64,
    pub name: Option<String>,
    pub bookmarked: bool,
}

pub fn list_sessions(conn: &Connection) -> Result<Vec<SessionRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, started_at, ended_at, car_ordinal, car_class, car_pi, best_lap, packet_count,
                name, bookmarked
         FROM sessions ORDER BY bookmarked DESC, started_at DESC LIMIT 100",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(SessionRow {
            id: r.get(0)?,
            started_at: r.get(1)?,
            ended_at: r.get(2)?,
            car_ordinal: r.get(3)?,
            car_class: r.get(4)?,
            car_pi: r.get(5)?,
            best_lap: r.get(6)?,
            packet_count: r.get(7)?,
            name: r.get(8)?,
            bookmarked: r.get::<_, i64>(9)? != 0,
        })
    })?;
    rows.collect()
}

pub fn rename_session(conn: &Connection, id: i64, name: Option<&str>) -> Result<()> {
    // Empty/whitespace name clears back to the default label.
    let trimmed = name.map(str::trim).filter(|s| !s.is_empty());
    conn.execute(
        "UPDATE sessions SET name=?1 WHERE id=?2",
        rusqlite::params![trimmed, id],
    )?;
    Ok(())
}

pub fn set_session_bookmark(conn: &Connection, id: i64, bookmarked: bool) -> Result<()> {
    conn.execute(
        "UPDATE sessions SET bookmarked=?1 WHERE id=?2",
        rusqlite::params![bookmarked as i64, id],
    )?;
    Ok(())
}

pub fn get_session_packets(conn: &Connection, session_id: i64) -> Result<Vec<Vec<u8>>> {
    let mut stmt = conn.prepare(
        "SELECT data FROM session_packets WHERE session_id=?1 ORDER BY timestamp_ms ASC",
    )?;
    let rows = stmt.query_map([session_id], |r| r.get::<_, Vec<u8>>(0))?;
    rows.collect()
}

pub fn delete_session(conn: &Connection, id: i64) -> Result<()> {
    // FK cascade isn't enabled on this connection, so clean children explicitly.
    conn.execute("DELETE FROM session_laps WHERE session_id=?1", [id])?;
    conn.execute("DELETE FROM session_packets WHERE session_id=?1", [id])?;
    conn.execute("DELETE FROM sessions WHERE id=?1", [id])?;
    Ok(())
}

pub fn clear_all_sessions(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "DELETE FROM session_laps; DELETE FROM session_packets; DELETE FROM sessions;",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn in_memory() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init(&conn).unwrap();
        conn
    }

    #[test]
    fn init_creates_tables() {
        let conn = in_memory();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('sessions','session_packets')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn open_and_close_session() {
        let conn = in_memory();
        let id = open_session(&conn, 12345, 3, 5, 900).unwrap();
        assert!(id > 0);
        close_session(&conn, id, 1000, 78.5).unwrap();
        let ended: Option<i64> = conn
            .query_row("SELECT ended_at FROM sessions WHERE id=?1", [id], |r| r.get(0))
            .unwrap();
        assert!(ended.is_some());
    }

    #[test]
    fn reopen_clears_ended_at() {
        let conn = in_memory();
        let id = open_session(&conn, 12345, 3, 5, 900).unwrap();
        close_session(&conn, id, 1000, 78.5).unwrap();
        reopen_session(&conn, id).unwrap();
        let ended: Option<i64> = conn
            .query_row("SELECT ended_at FROM sessions WHERE id=?1", [id], |r| r.get(0))
            .unwrap();
        assert!(ended.is_none());
    }

    #[test]
    fn close_records_caller_supplied_best() {
        // best_lap is now the authoritative MIN of the lap table supplied by
        // the caller, so the latest close with a real value wins (this is what
        // corrects a provisional partial after a rewind).
        let conn = in_memory();
        let id = open_session(&conn, 0, 0, 0, 0).unwrap();
        close_session(&conn, id, 100, 37.0).unwrap(); // provisional partial
        reopen_session(&conn, id).unwrap();
        close_session(&conn, id, 200, 54.0).unwrap(); // true min after upsert
        let best: Option<f32> = conn
            .query_row("SELECT best_lap FROM sessions WHERE id=?1", [id], |r| r.get(0))
            .unwrap();
        assert_eq!(best, Some(54.0));
    }

    #[test]
    fn close_updates_to_better_best_lap() {
        let conn = in_memory();
        let id = open_session(&conn, 0, 0, 0, 0).unwrap();
        close_session(&conn, id, 100, 65.5).unwrap();
        reopen_session(&conn, id).unwrap();
        // Better lap after rewind — should update
        close_session(&conn, id, 200, 60.0).unwrap();
        let best: Option<f32> = conn
            .query_row("SELECT best_lap FROM sessions WHERE id=?1", [id], |r| r.get(0))
            .unwrap();
        assert_eq!(best, Some(60.0));
    }

    #[test]
    fn close_with_no_lap_keeps_existing_best() {
        let conn = in_memory();
        let id = open_session(&conn, 0, 0, 0, 0).unwrap();
        close_session(&conn, id, 100, 65.5).unwrap();
        reopen_session(&conn, id).unwrap();
        // -1.0 means no lap was recorded post-rewind
        close_session(&conn, id, 200, -1.0).unwrap();
        let best: Option<f32> = conn
            .query_row("SELECT best_lap FROM sessions WHERE id=?1", [id], |r| r.get(0))
            .unwrap();
        assert_eq!(best, Some(65.5));
    }

    #[test]
    fn migrate_is_idempotent() {
        let conn = in_memory();
        // init() already ran migrate once; running again must not error.
        migrate(&conn);
        migrate(&conn);
        let id = open_session(&conn, 1, 0, 0, 0).unwrap();
        let rows = list_sessions(&conn).unwrap();
        let row = rows.iter().find(|r| r.id == id).unwrap();
        assert_eq!(row.name, None);
        assert!(!row.bookmarked);
    }

    #[test]
    fn rename_and_clear_session() {
        let conn = in_memory();
        let id = open_session(&conn, 1, 0, 0, 0).unwrap();
        rename_session(&conn, id, Some("  Nürburgring hotlap  ")).unwrap();
        let name: Option<String> = conn
            .query_row("SELECT name FROM sessions WHERE id=?1", [id], |r| r.get(0))
            .unwrap();
        assert_eq!(name.as_deref(), Some("Nürburgring hotlap"));
        // Whitespace-only clears back to default (NULL).
        rename_session(&conn, id, Some("   ")).unwrap();
        let cleared: Option<String> = conn
            .query_row("SELECT name FROM sessions WHERE id=?1", [id], |r| r.get(0))
            .unwrap();
        assert_eq!(cleared, None);
    }

    #[test]
    fn bookmark_toggle_and_ordering() {
        let conn = in_memory();
        let a = open_session(&conn, 100, 0, 0, 0).unwrap();
        let b = open_session(&conn, 200, 0, 0, 0).unwrap();
        set_session_bookmark(&conn, a, true).unwrap();
        let rows = list_sessions(&conn).unwrap();
        // Bookmarked session sorts first despite older started_at.
        assert_eq!(rows[0].id, a);
        assert!(rows[0].bookmarked);
        assert_eq!(rows[1].id, b);
        set_session_bookmark(&conn, a, false).unwrap();
        let rows = list_sessions(&conn).unwrap();
        assert_eq!(rows[0].id, b);
    }

    #[test]
    fn laps_insert_get_and_replace() {
        let conn = in_memory();
        let id = open_session(&conn, 1, 0, 0, 0).unwrap();
        insert_lap(&conn, id, 1, 92.5).unwrap();
        insert_lap(&conn, id, 2, 90.1).unwrap();
        // Re-driven lap 2 (rewind) overwrites, no duplicate row.
        insert_lap(&conn, id, 2, 88.0).unwrap();
        let laps = get_session_laps(&conn, id).unwrap();
        assert_eq!(laps.len(), 2);
        assert_eq!(laps[0].lap_number, 1);
        assert!((laps[1].lap_time - 88.0).abs() < 0.001);
    }

    #[test]
    fn delete_session_removes_laps_and_packets() {
        let conn = in_memory();
        let id = open_session(&conn, 1, 0, 0, 0).unwrap();
        insert_lap(&conn, id, 1, 90.0).unwrap();
        insert_packet(&conn, id, 1, &vec![0u8; 311]).unwrap();
        delete_session(&conn, id).unwrap();
        assert_eq!(get_session_laps(&conn, id).unwrap().len(), 0);
        let pkts: i64 = conn
            .query_row("SELECT COUNT(*) FROM session_packets WHERE session_id=?1", [id], |r| r.get(0))
            .unwrap();
        assert_eq!(pkts, 0);
    }

    #[test]
    fn clear_all_sessions_empties_everything() {
        let conn = in_memory();
        for s in 0..3 {
            let id = open_session(&conn, s, 0, 0, 0).unwrap();
            insert_lap(&conn, id, 1, 80.0).unwrap();
            insert_packet(&conn, id, 1, &vec![0u8; 311]).unwrap();
        }
        clear_all_sessions(&conn).unwrap();
        assert_eq!(list_sessions(&conn).unwrap().len(), 0);
        let counts: (i64, i64) = conn
            .query_row(
                "SELECT (SELECT COUNT(*) FROM session_packets), (SELECT COUNT(*) FROM session_laps)",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(counts, (0, 0));
    }

    #[test]
    fn insert_and_count_packets() {
        let conn = in_memory();
        let id = open_session(&conn, 0, 0, 0, 0).unwrap();
        let blob = vec![0u8; 311];
        insert_packet(&conn, id, 1000, &blob).unwrap();
        insert_packet(&conn, id, 2000, &blob).unwrap();
        let count: i64 = conn
            .query_row("SELECT packet_count FROM sessions WHERE id=?1", [id], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 2);
    }
}
