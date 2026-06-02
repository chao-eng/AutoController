/// Grace window: a new stream starting within this many ms of the last close,
/// with race time that went backwards, is treated as a rewind of the same session.
const REWIND_WINDOW_MS: u64 = 30_000;
/// Rewinds to the very start are indistinguishable from a new race; only treat
/// mid-race values (> 5 s) as rewind candidates.
const REWIND_MIN_RACE_TIME: f32 = 5.0;

pub enum SessionAction {
    Open { car_ordinal: i32, car_class: i32, car_pi: i32 },
    Close,
    None,
}

/// A lap that just finished and should be persisted.
pub struct CompletedLap {
    pub lap_number: i64,
    pub lap_time: f32,
}

pub struct SessionManager {
    auto_record: bool,
    active_id: Option<i64>,
    best_lap: f32,
    last_race_time: f32,
    // Highest progress time seen this session; the rewind baseline. Tracking
    // the peak (not the latest) means a rewind that scrubs time backward
    // during the close grace window still reads as "went backward".
    peak_race_time: f32,
    // Lap tracking for the active session. Driven by current_lap (the live
    // lap clock): Forza does NOT emit last_lap for the final lap of a race,
    // so last_lap is unusable. A lap completes when current_lap resets to ~0
    // while still racing; its time is the peak current_lap reached.
    prev_current_lap: f32,
    cur_lap_peak: f32,
    prev_race_time: f32,
    rewind_guard: u32,
    laps_recorded: i64,
    // Packets recorded this session — used to keep the discard-empty rule from
    // throwing away long lapless runs (point-to-point / sprint races).
    ticks: u64,
    // Rewind detection state — set on close, consumed on reopen
    closed_id: Option<i64>,
    closed_wall_ms: u64,
    last_race_time_at_close: f32,
}

impl SessionManager {
    pub fn new(auto_record: bool) -> Self {
        Self {
            auto_record,
            active_id: Option::None,
            best_lap: f32::MAX,
            last_race_time: 0.0,
            peak_race_time: 0.0,
            prev_current_lap: 0.0,
            cur_lap_peak: 0.0,
            prev_race_time: 0.0,
            rewind_guard: 0,
            laps_recorded: 0,
            ticks: 0,
            closed_id: None,
            closed_wall_ms: 0,
            last_race_time_at_close: 0.0,
        }
    }

    pub fn active_session_id(&self) -> Option<i64> {
        self.active_id
    }

    pub fn set_auto_record(&mut self, v: bool) {
        self.auto_record = v;
    }

    /// Only changes which session is active. Lap-tracking state is NOT reset
    /// here — a rewind closes then reopens the SAME session, and that state
    /// must survive the gap. Use `begin_new_session` for a genuinely new race.
    pub fn set_active_id(&mut self, id: Option<i64>) {
        self.active_id = id;
    }

    /// Reset all per-race tracking. Called only when a brand-new session is
    /// opened (not on a rewind reopen, which continues the same race).
    pub fn begin_new_session(&mut self) {
        self.best_lap = f32::MAX;
        self.prev_current_lap = 0.0;
        self.cur_lap_peak = 0.0;
        self.prev_race_time = 0.0;
        self.rewind_guard = 0;
        self.laps_recorded = 0;
        self.ticks = 0;
        self.peak_race_time = 0.0;
    }

    pub fn update_best_lap(&mut self, lap: f32) {
        if lap > 0.0 && lap < self.best_lap {
            self.best_lap = lap;
        }
    }

    /// The best lap to persist on close: -1.0 means "no lap recorded", which
    /// `db::close_session` treats as "keep the existing best" (rewind-safe).
    pub fn best_for_close(&self) -> f32 {
        if self.best_lap == f32::MAX { -1.0 } else { self.best_lap }
    }

    /// Minimum running time before a `current_lap` reset counts as a real lap.
    /// The rolling start / countdown makes `current_lap` tick a few seconds
    /// and then reset to 0 at the actual start line; that must not register as
    /// a (very short) lap — it also poisons the best lap. Real FH6 circuit
    /// laps are far longer than this.
    const MIN_LAP_SECS: f32 = 20.0;

    pub fn laps_recorded(&self) -> i64 {
        self.laps_recorded
    }

    pub fn ticks(&self) -> u64 {
        self.ticks
    }

    /// Ticks to suppress lap completion after a rewind is detected — a rewind
    /// scrubs current_lap down through ~0, which would otherwise look exactly
    /// like a finish-line crossing and record a bogus short lap.
    const REWIND_GUARD_TICKS: u32 = 60;

    /// Feed every in-event tick. A lap completes when `current_lap` drops back
    /// to ~0 **while still racing** (a line crossing); its time is the peak
    /// `current_lap` reached. A rewind is distinguished from a real lap
    /// boundary because it also drives the cumulative `race_time` backward
    /// and/or briefly drops `is_race_on` — either arms a guard that suppresses
    /// false completions while the clock is scrubbing.
    pub fn note_tick(
        &mut self,
        is_race_on: bool,
        current_lap: f32,
        race_time: f32,
    ) -> Option<CompletedLap> {
        self.ticks += 1;
        if current_lap > self.cur_lap_peak {
            self.cur_lap_peak = current_lap;
        }

        // Rewind indicators: paused/rewinding stream, or the cumulative race
        // clock jumped backward. (race_time is 0 in Time Trial — there the
        // is_race_on drop is the signal.)
        if !is_race_on
            || (race_time > 0.0 && race_time + 0.25 < self.prev_race_time)
        {
            self.rewind_guard = Self::REWIND_GUARD_TICKS;
        }
        if race_time > 0.0 {
            self.prev_race_time = race_time;
        }

        let completed = if is_race_on
            && self.rewind_guard == 0
            && self.prev_current_lap > Self::MIN_LAP_SECS
            && current_lap < 1.0
        {
            let t = self.cur_lap_peak;
            self.cur_lap_peak = current_lap; // next lap starts now
            let idx = self.laps_recorded;
            self.laps_recorded += 1;
            self.update_best_lap(t);
            Some(CompletedLap { lap_number: idx, lap_time: t })
        } else {
            None
        };

        if self.rewind_guard > 0 && is_race_on {
            self.rewind_guard -= 1;
        }
        self.prev_current_lap = current_lap;
        completed
    }

    /// Call on close. The final lap of a race ends with the race ending (no
    /// line-crossing reset), so the in-progress lap is recorded here — only if
    /// it's lap-sized, which rejects a post-race cool-down roll.
    ///
    /// Non-destructive: it does NOT consume the peak or advance the lap index.
    /// A rewind close/reopen keeps the same session, so a later close (or a
    /// real completion) re-emits the SAME lap index and the DB upsert
    /// overwrites this provisional value with the true (longer) lap.
    pub fn finalize_final_lap(&mut self) -> Option<CompletedLap> {
        let t = self.cur_lap_peak;
        let floor = if self.best_lap == f32::MAX {
            10.0
        } else {
            (0.5 * self.best_lap).max(10.0)
        };
        if t >= floor {
            self.update_best_lap(t);
            Some(CompletedLap { lap_number: self.laps_recorded, lap_time: t })
        } else {
            None
        }
    }

    pub fn update_race_time(&mut self, t: f32) {
        self.last_race_time = t;
        if t > self.peak_race_time {
            self.peak_race_time = t;
        }
    }

    /// Call when a session is about to close. Stashes state needed to detect a
    /// subsequent rewind within REWIND_WINDOW_MS.
    pub fn note_close(&mut self, wall_ms: u64) {
        self.closed_id = self.active_id;
        self.closed_wall_ms = wall_ms;
        self.last_race_time_at_close = self.peak_race_time;
    }

    /// Returns the session id to reopen when `new_race_time` went backward
    /// within the rewind window. Consumes the stashed state so it cannot fire twice.
    pub fn check_reopen(&mut self, new_race_time: f32, now_wall_ms: u64) -> Option<i64> {
        let id = self.closed_id?;
        let gap_ms = now_wall_ms.saturating_sub(self.closed_wall_ms);
        if gap_ms < REWIND_WINDOW_MS
            && new_race_time > REWIND_MIN_RACE_TIME
            && new_race_time < self.last_race_time_at_close
        {
            self.closed_id = None;
            Some(id)
        } else {
            None
        }
    }

    pub fn on_race_on_change(
        &mut self,
        was_racing: bool,
        is_racing: bool,
        car_ordinal: i32,
        car_class: i32,
        car_pi: i32,
    ) -> SessionAction {
        match (was_racing, is_racing) {
            (false, true) if self.auto_record => SessionAction::Open { car_ordinal, car_class, car_pi },
            (true, false) if self.active_id.is_some() => SessionAction::Close,
            _ => SessionAction::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_session_when_not_racing() {
        let sm = SessionManager::new(true);
        assert!(sm.active_session_id().is_none());
    }

    #[test]
    fn opens_session_on_race_start() {
        let mut sm = SessionManager::new(true);
        let action = sm.on_race_on_change(false, true, 99, 3, 800);
        assert!(matches!(action, SessionAction::Open { car_ordinal: 99, .. }));
    }

    #[test]
    fn closes_session_on_race_end() {
        let mut sm = SessionManager::new(true);
        sm.on_race_on_change(false, true, 0, 0, 0);
        sm.set_active_id(Some(1));
        let action = sm.on_race_on_change(true, false, 0, 0, 0);
        assert!(matches!(action, SessionAction::Close));
    }

    #[test]
    fn lap_completes_on_current_lap_reset_while_racing() {
        let mut sm = SessionManager::new(true);
        sm.set_active_id(Some(1));
        // race_time is cumulative and ever-increasing across laps.
        assert!(sm.note_tick(true, 5.0, 5.0).is_none());
        assert!(sm.note_tick(true, 57.2, 57.2).is_none()); // peak builds
        let l1 = sm.note_tick(true, 0.43, 57.4).expect("lap 1 (line crossed)");
        assert_eq!(l1.lap_number, 0);
        assert!((l1.lap_time - 57.2).abs() < 0.001);
        assert!(sm.note_tick(true, 30.0, 87.0).is_none());
        assert!(sm.note_tick(true, 55.2, 112.4).is_none());
        let l2 = sm.note_tick(true, 0.5, 112.6).expect("lap 2");
        assert_eq!(l2.lap_number, 1);
        assert!((l2.lap_time - 55.2).abs() < 0.001);
    }

    #[test]
    fn rewind_scrub_does_not_record_a_short_lap() {
        // Mid-lap rewind scrubs current_lap down through ~0 while the
        // cumulative race clock jumps backward — must NOT be a lap. The lap
        // then continues and completes at its true (peak) time.
        let mut sm = SessionManager::new(true);
        sm.set_active_id(Some(1));
        sm.note_tick(true, 20.0, 20.0);
        sm.note_tick(true, 45.0, 45.0); // peak 45 so far
        // Rewind: brief is_race_on=0 blip, race clock + current_lap backward.
        sm.note_tick(false, 0.0, 0.0);
        // current_lap scrubs down past ~0 while race_time is well behind 45.
        assert!(sm.note_tick(true, 0.5, 30.0).is_none(), "scrub must not be a lap");
        assert_eq!(sm.laps_recorded(), 0);
        // Re-drive the rest of the lap over many ticks (the rewind guard
        // expires well within the seconds it takes to finish the lap).
        let mut t = 30.0_f32;
        for i in 1..=80 {
            let cl = 0.5 + i as f32 * 0.67; // climbs to ~54 over 80 ticks
            t += 0.67;
            assert!(sm.note_tick(true, cl, t).is_none());
        }
        let lap = sm.note_tick(true, 0.4, t + 0.2).expect("real lap completion");
        assert_eq!(lap.lap_number, 0);
        // Peak reached ~54 — the full lap, not the 45 pre-rewind value.
        assert!(lap.lap_time > 53.0 && lap.lap_time < 55.0, "got {}", lap.lap_time);
    }

    #[test]
    fn final_race_lap_finalized_on_close() {
        let mut sm = SessionManager::new(true);
        sm.set_active_id(Some(1));
        sm.note_tick(true, 57.2, 57.2);
        sm.note_tick(true, 0.4, 57.4);
        sm.note_tick(true, 55.2, 112.6);
        sm.note_tick(true, 0.4, 112.8);
        // Final lap climbs, then the race ends (is_race_on=0, no reset).
        sm.note_tick(true, 40.0, 152.0);
        sm.note_tick(true, 53.6, 165.6);
        assert!(sm.note_tick(false, 0.0, 0.0).is_none());
        let f = sm.finalize_final_lap().expect("final lap recorded");
        assert!((f.lap_time - 53.6).abs() < 0.001);
        assert!((sm.best_for_close() - 53.6).abs() < 0.001);
    }

    #[test]
    fn rewind_stitch_preserves_lap_state_across_close_reopen() {
        // A rewind closes then reopens the SAME session. Lap index, peak and
        // best must survive the gap so laps aren't renumbered/overwritten.
        let mut sm = SessionManager::new(true);
        sm.begin_new_session();
        sm.set_active_id(Some(5));
        sm.note_tick(true, 50.0, 50.0);
        let l0 = sm.note_tick(true, 0.4, 50.2).expect("lap 0");
        assert_eq!(l0.lap_number, 0);
        // Mid lap 1, a long rewind triggers a session close.
        sm.note_tick(true, 30.0, 80.0);
        let prov = sm.finalize_final_lap().expect("provisional final");
        assert_eq!(prov.lap_number, 1);
        sm.set_active_id(None); // close — must NOT wipe lap state
        assert_eq!(sm.laps_recorded(), 1);
        // Rewind reopen of the same session; lap 1 continues to its true time.
        sm.set_active_id(Some(5));
        sm.note_tick(true, 55.0, 105.0);
        let l1 = sm.note_tick(true, 0.5, 105.2).expect("lap 1 true completion");
        assert_eq!(l1.lap_number, 1); // same index → DB upsert overwrites
        assert!((l1.lap_time - 55.0).abs() < 0.001);
        assert_eq!(sm.laps_recorded(), 2);
    }

    #[test]
    fn post_race_cooldown_is_not_finalized() {
        let mut sm = SessionManager::new(true);
        sm.set_active_id(Some(1));
        sm.note_tick(true, 53.0, 53.0);
        let l = sm.note_tick(true, 0.4, 53.2).expect("real final lap (crossed line)");
        assert!((l.lap_time - 53.0).abs() < 0.001);
        sm.note_tick(true, 1.5, 54.5);
        sm.note_tick(true, 3.8, 56.8);
        assert!(sm.finalize_final_lap().is_none());
        assert!((sm.best_for_close() - 53.0).abs() < 0.001);
    }

    #[test]
    fn no_action_when_race_on_unchanged() {
        let mut sm = SessionManager::new(true);
        let action = sm.on_race_on_change(true, true, 0, 0, 0);
        assert!(matches!(action, SessionAction::None));
    }

    #[test]
    fn disabled_auto_record_never_opens() {
        let mut sm = SessionManager::new(false);
        let action = sm.on_race_on_change(false, true, 0, 0, 0);
        assert!(matches!(action, SessionAction::None));
    }

    #[test]
    fn rewind_reopens_session_within_window() {
        let mut sm = SessionManager::new(true);
        sm.set_active_id(Some(42));
        sm.update_race_time(90.0);
        sm.note_close(1_000_000);
        sm.set_active_id(None);
        // New stream starts at 60 s — time went backward, within 30 s wall gap
        let reopen = sm.check_reopen(60.0, 1_005_000);
        assert_eq!(reopen, Some(42));
    }

    #[test]
    fn no_reopen_after_long_gap() {
        let mut sm = SessionManager::new(true);
        sm.set_active_id(Some(7));
        sm.update_race_time(90.0);
        sm.note_close(0);
        sm.set_active_id(None);
        // 60 s gap — beyond the window
        let reopen = sm.check_reopen(60.0, 60_001);
        assert!(reopen.is_none());
    }

    #[test]
    fn no_reopen_for_fresh_start() {
        let mut sm = SessionManager::new(true);
        sm.set_active_id(Some(5));
        sm.update_race_time(120.0);
        sm.note_close(0);
        sm.set_active_id(None);
        // Race time near zero — looks like a new race, not a rewind
        let reopen = sm.check_reopen(1.0, 2_000);
        assert!(reopen.is_none());
    }

    #[test]
    fn no_reopen_when_time_advances() {
        let mut sm = SessionManager::new(true);
        sm.set_active_id(Some(3));
        sm.update_race_time(45.0);
        sm.note_close(0);
        sm.set_active_id(None);
        // Race time went forward — not a rewind
        let reopen = sm.check_reopen(50.0, 5_000);
        assert!(reopen.is_none());
    }

    #[test]
    fn rewind_during_grace_window_still_reopens() {
        let mut sm = SessionManager::new(true);
        sm.set_active_id(Some(11));
        sm.update_race_time(90.0); // peak
        // Rewind scrubs the timer back while the close grace period runs.
        sm.update_race_time(8.0);
        sm.note_close(1_000);
        sm.set_active_id(None);
        // New stream resumes at the rewound time — baseline is the peak (90),
        // so this is still recognised as a rewind, not a fresh session.
        let reopen = sm.check_reopen(9.0, 2_000);
        assert_eq!(reopen, Some(11));
    }

    #[test]
    fn check_reopen_consumes_closed_id() {
        let mut sm = SessionManager::new(true);
        sm.set_active_id(Some(9));
        sm.update_race_time(80.0);
        sm.note_close(0);
        sm.set_active_id(None);
        // First call succeeds
        assert!(sm.check_reopen(40.0, 1_000).is_some());
        // Second call returns None — closed_id was consumed
        assert!(sm.check_reopen(40.0, 2_000).is_none());
    }
}
