use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tokio::sync::{broadcast, mpsc};

use super::{db, event::ServerEvent, parser, session::SessionAction, AppState};

const SESSION_QUEUE_CAPACITY: usize = 512;
const CLOSE_GRACE: u32 = 150;

struct SessionIngest {
    pkt: parser::TelemetryPacket,
    raw: Vec<u8>,
}

pub async fn run(state: Arc<AppState>, port: u16, tx: broadcast::Sender<ServerEvent>) {
    let addr = format!("0.0.0.0:{port}");
    let socket = match UdpSocket::bind(&addr).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(target: "fh6_telemetry::udp", address = %addr, error = %e, "failed to bind UDP socket");
            let _ = tx.send(ServerEvent::BindFailed(format!(
                "Cannot bind port {port}: {e}"
            )));
            return;
        }
    };
    tracing::info!(target: "fh6_telemetry::udp", address = %addr, "listening for telemetry packets");

    let session_tx = start_session_writer(state, tx.clone());
    let mut buf = vec![0u8; 1024];
    let mut debug_logged = false;
    let mut dropped_session_packets: u64 = 0;
    let mut writer_closed_logged = false;

    loop {
        let (len, _) = match socket.recv_from(&mut buf).await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(target: "fh6_telemetry::udp", error = %e, "failed to receive UDP packet");
                continue;
            }
        };

        let raw = &buf[..len];

        if !debug_logged {
            debug_logged = true;
            tracing::info!(target: "fh6_telemetry::udp", bytes = len, "received first telemetry packet");
            if raw.len() >= 323 {
                let speed = f32::from_le_bytes(raw[256..260].try_into().unwrap_or([0; 4]));
                let thr = raw[315];
                let brk = raw[316];
                let gear = raw[319];
                let pos = raw[314];
                let tire_f_raw = f32::from_le_bytes(raw[268..272].try_into().unwrap_or([0; 4]));
                tracing::debug!(
                    target: "fh6_telemetry::udp",
                    speed_ms = speed,
                    throttle = thr,
                    brake = brk,
                    gear = gear,
                    race_position = pos,
                    tire_fl_raw = tire_f_raw,
                    "first packet sample"
                );
            }
        }

        let pkt = match parser::parse(raw) {
            Ok(p) => p,
            Err(_) => continue,
        };

        // 更新全局遥测变量内存状态
        super::update_last_telemetry(&pkt);

        // Always emit live data regardless of session recording state.
        let _ = tx.send(ServerEvent::Tick(pkt.clone()));

        let ingest = SessionIngest {
            pkt,
            raw: raw.to_vec(),
        };
        match session_tx.try_send(ingest) {
            Ok(()) => {}
            Err(mpsc::error::TrySendError::Full(_)) => {
                dropped_session_packets = dropped_session_packets.saturating_add(1);
                if dropped_session_packets == 1 || dropped_session_packets % 100 == 0 {
                    tracing::warn!(
                        target: "fh6_telemetry::session",
                        dropped_packets = dropped_session_packets,
                        "session writer queue is full; dropping telemetry packet for recording"
                    );
                }
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                if !writer_closed_logged {
                    writer_closed_logged = true;
                    tracing::error!(
                        target: "fh6_telemetry::session",
                        "session writer is closed; live telemetry will continue without recording"
                    );
                }
            }
        }
    }
}

fn start_session_writer(
    state: Arc<AppState>,
    tx: broadcast::Sender<ServerEvent>,
) -> mpsc::Sender<SessionIngest> {
    let (session_tx, mut session_rx) = mpsc::channel::<SessionIngest>(SESSION_QUEUE_CAPACITY);

    if let Err(e) = std::thread::Builder::new()
        .name("fh6-telemetry-writer".to_string())
        .spawn(move || {
            let mut prev_in_event = false;
            let mut close_pending: u32 = 0;

            while let Some(ingest) = session_rx.blocking_recv() {
                // Record whenever a lap is being timed: races/Rivals
                // (race_position > 0) and Time Trial (race_position 0 but the
                // lap clock runs). Free-roam has no lap timer so it stays
                // unrecorded. Grace period stops pause-menu packets from
                // splitting a session.
                let timed_lap = ingest.pkt.current_lap > 0.0;
                let raw_in_event =
                    ingest.pkt.is_race_on && (ingest.pkt.race_position > 0 || timed_lap);
                if raw_in_event {
                    close_pending = 0;
                } else {
                    close_pending = close_pending.saturating_add(1);
                }
                let in_event = raw_in_event || close_pending < CLOSE_GRACE;

                handle_session(
                    &state,
                    &tx,
                    &ingest.pkt,
                    &ingest.raw,
                    prev_in_event,
                    in_event,
                );
                prev_in_event = in_event;
            }

            tracing::info!(target: "fh6_telemetry::session", "session writer stopped");
        })
    {
        tracing::error!(target: "fh6_telemetry::session", error = %e, "failed to start session writer");
    }

    session_tx
}

fn handle_session(
    state: &AppState,
    tx: &broadcast::Sender<ServerEvent>,
    pkt: &parser::TelemetryPacket,
    raw: &[u8],
    prev_in_event: bool,
    in_event: bool,
) {
    let mut sm = state.session_manager.lock().unwrap();
    let db = state.db.lock().unwrap();

    let now_ms: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    // Progress timer for rewind detection. Races/Rivals report a cumulative
    // current_race_time; Time Trial leaves it at 0 and only the lap clock
    // advances — fall back to that so rewinds still stitch the session.
    let progress = if pkt.current_race_time > 0.0 {
        pkt.current_race_time
    } else {
        pkt.current_lap
    };

    // Apply event transition before inserting so the opening packet is captured
    let action = sm.on_race_on_change(
        prev_in_event,
        in_event,
        pkt.car_ordinal,
        pkt.car_class,
        pkt.car_pi,
    );

    // Open/reopen first so the opening packet is captured below.
    if let SessionAction::Open {
        car_ordinal,
        car_class,
        car_pi,
    } = &action
    {
        let (car_ordinal, car_class, car_pi) = (*car_ordinal, *car_class, *car_pi);
        // Check if the new stream looks like a rewind into the previous
        // session: race time went backward within the rewind window.
        if let Some(reopen_id) = sm.check_reopen(progress, now_ms) {
            match db::reopen_session(&db, reopen_id) {
                Ok(()) => {
                    sm.set_active_id(Some(reopen_id));
                    tracing::info!(target: "fh6_telemetry::session", session_id = reopen_id, "rewind detected, continuing session");
                }
                Err(e) => {
                    tracing::warn!(target: "fh6_telemetry::session", session_id = reopen_id, error = %e, "failed to reopen session")
                }
            }
        } else {
            match db::open_session(&db, now_ms as i64, car_ordinal, car_class, car_pi) {
                Ok(id) => {
                    // Genuinely new race — reset all lap tracking. (A rewind
                    // reopen above deliberately does NOT, to continue the run.)
                    sm.begin_new_session();
                    sm.set_active_id(Some(id));
                    tracing::info!(target: "fh6_telemetry::session", session_id = id, "opened session");
                }
                Err(e) => {
                    tracing::error!(target: "fh6_telemetry::session", error = %e, "failed to open session");
                    let _ = tx.send(ServerEvent::SessionError(format!(
                        "Failed to open session: {e}"
                    )));
                }
            }
        }
    }

    // Record this packet while the session is still active.
    if let Some(session_id) = sm.active_session_id() {
        // Best lap is derived only from laps we time ourselves — Forza's
        // best_lap field carries stale/garbage values across sessions.
        sm.update_race_time(progress);
        if let Some(lap) = sm.note_tick(pkt.is_race_on, pkt.current_lap, pkt.current_race_time) {
            if let Err(e) = db::insert_lap(&db, session_id, lap.lap_number, lap.lap_time) {
                tracing::warn!(target: "fh6_telemetry::session", session_id, lap_number = lap.lap_number, error = %e, "failed to insert lap");
            }
        }
        if let Err(e) = db::insert_packet(&db, session_id, pkt.timestamp_ms, raw) {
            tracing::error!(target: "fh6_telemetry::session", session_id, error = %e, "failed to insert telemetry packet");
            let _ = tx.send(ServerEvent::SessionError(format!(
                "Failed to write telemetry: {e}"
            )));
        }
        // Lazily fill car metadata: the opening packet sometimes arrives before the
        // game has populated car_ordinal. This no-ops once car_ordinal is non-zero.
        if pkt.car_ordinal != 0 {
            db::update_session_car_if_unknown(
                &db,
                session_id,
                pkt.car_ordinal,
                pkt.car_class,
                pkt.car_pi,
            )
            .ok();
        }
    }

    // Close after recording.
    if matches!(action, SessionAction::Close) {
        if let Some(id) = sm.active_session_id() {
            sm.note_close(now_ms);
            // The final lap ends with the race ending (no line-crossing
            // reset), so record the in-progress lap here. Non-destructive: a
            // rewind reopen continues the same lap and a later close overwrites
            // this provisional value with the true (longer) one.
            let final_lap = sm.finalize_final_lap();
            if let Some(lap) = &final_lap {
                if let Err(e) = db::insert_lap(&db, id, lap.lap_number, lap.lap_time) {
                    tracing::warn!(target: "fh6_telemetry::session", session_id = id, lap_number = lap.lap_number, error = %e, "failed to insert final lap");
                }
            }
            // Discard only a *tiny* lapless session (a pre-race / aborted
            // fragment, ~10s). A longer lapless run is a real point-to-point
            // / sprint race and must be kept. ~400 packets ≈ 10s.
            if sm.laps_recorded() == 0 && final_lap.is_none() && sm.ticks() < 400 {
                if let Err(e) = db::delete_session(&db, id) {
                    tracing::warn!(target: "fh6_telemetry::session", session_id = id, error = %e, "failed to discard empty session");
                } else {
                    tracing::info!(target: "fh6_telemetry::session", session_id = id, "discarded empty session");
                }
            } else {
                // Best is the fastest lap actually in the table (rewind
                // upserts already corrected); -1.0 = none → keep existing.
                let best = db::min_lap_time(&db, id).ok().flatten().unwrap_or(-1.0);
                if let Err(e) = db::close_session(&db, id, now_ms as i64, best) {
                    tracing::error!(target: "fh6_telemetry::session", session_id = id, error = %e, "failed to close session");
                    let _ = tx.send(ServerEvent::SessionError(format!(
                        "Failed to close session: {e}"
                    )));
                } else {
                    tracing::info!(target: "fh6_telemetry::session", session_id = id, "closed session");
                }
            }
        }
        sm.set_active_id(None);
    }
}
