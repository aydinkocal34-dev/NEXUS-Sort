//! Pure life-regeneration and level-progress rules for NEXUS Sort.
//! Persistence and Google Play billing are deliberately separate concerns.

pub const MAX_LIVES: u8 = 5;
pub const STARTING_LIVES: u8 = 5;
pub const REFILL_INTERVAL_MS: u64 = 30 * 60 * 1000;
pub const MAX_REASONABLE_OFFLINE_MS: u64 = 30 * 24 * 60 * 60 * 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockAnomaly {
    None,
    Backwards,
    TooFarForward,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefillOutcome {
    pub lives_before: u8,
    pub lives_after: u8,
    pub ticks_applied: u32,
    pub pending_ms_after: u64,
    pub clock_anomaly: ClockAnomaly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelOutcome {
    Success,
    Failure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerState {
    pub lives: u8,
    /// Unix epoch milliseconds. Device time is not a trusted anti-cheat clock.
    pub last_refill_at_ms: u64,
    /// Partial progress toward the next 30-minute refill.
    pub pending_ms: u64,
    /// Highest consecutively unlocked/completed level index.
    pub level_progress: u32,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            lives: STARTING_LIVES,
            last_refill_at_ms: 0,
            pending_ms: 0,
            level_progress: 0,
        }
    }
}

impl PlayerState {
    pub fn new(now_ms: u64) -> Self {
        Self {
            last_refill_at_ms: now_ms,
            ..Self::default()
        }
    }

    pub fn clamp_lives(&mut self) {
        self.lives = self.lives.min(MAX_LIVES);
    }

    /// Applies elapsed time. Clock rollback is ignored without moving the saved
    /// timestamp backwards; a jump beyond 30 days is flagged and ignored.
    pub fn apply_refill(&mut self, now_ms: u64) -> RefillOutcome {
        self.clamp_lives();
        let before = self.lives;

        if self.lives >= MAX_LIVES {
            self.pending_ms = 0;
            self.last_refill_at_ms = now_ms;
            return RefillOutcome {
                lives_before: before,
                lives_after: self.lives,
                ticks_applied: 0,
                pending_ms_after: 0,
                clock_anomaly: ClockAnomaly::None,
            };
        }

        if now_ms < self.last_refill_at_ms {
            return RefillOutcome {
                lives_before: before,
                lives_after: self.lives,
                ticks_applied: 0,
                pending_ms_after: self.pending_ms,
                clock_anomaly: ClockAnomaly::Backwards,
            };
        }

        let elapsed = now_ms - self.last_refill_at_ms;
        if elapsed > MAX_REASONABLE_OFFLINE_MS {
            // Reject implausible elapsed time without granting free lives, but
            // rebase the timestamp so one anomaly cannot lock refills forever.
            self.last_refill_at_ms = now_ms;
            self.pending_ms = 0;
            return RefillOutcome {
                lives_before: before,
                lives_after: self.lives,
                ticks_applied: 0,
                pending_ms_after: self.pending_ms,
                clock_anomaly: ClockAnomaly::TooFarForward,
            };
        }

        let total = self.pending_ms.saturating_add(elapsed);
        let ticks = total / REFILL_INTERVAL_MS;
        let room = u64::from(MAX_LIVES - self.lives);
        let gained = ticks.min(room);
        self.lives = self.lives.saturating_add(gained as u8).min(MAX_LIVES);
        self.pending_ms = if self.lives >= MAX_LIVES {
            0
        } else {
            total.saturating_sub(gained.saturating_mul(REFILL_INTERVAL_MS))
        };
        self.last_refill_at_ms = now_ms;

        RefillOutcome {
            lives_before: before,
            lives_after: self.lives,
            ticks_applied: gained as u32,
            pending_ms_after: self.pending_ms,
            clock_anomaly: ClockAnomaly::None,
        }
    }

    /// A failure costs one life. If lives were already partial, the existing
    /// refill timer is preserved so repeated failures cannot erase elapsed time.
    pub fn apply_level_outcome(&mut self, outcome: LevelOutcome, now_ms: u64) {
        match outcome {
            LevelOutcome::Success => {
                self.level_progress = self.level_progress.saturating_add(1);
            }
            LevelOutcome::Failure if self.lives > 0 => {
                let was_full = self.lives == MAX_LIVES;
                self.lives -= 1;
                if was_full {
                    self.last_refill_at_ms = now_ms;
                    self.pending_ms = 0;
                }
            }
            LevelOutcome::Failure => {}
        }
        self.clamp_lives();
    }

    pub fn remaining_refill_ms(&self) -> u64 {
        if self.lives >= MAX_LIVES {
            return 0;
        }
        REFILL_INTERVAL_MS.saturating_sub(self.pending_ms.min(REFILL_INTERVAL_MS))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_player_starts_with_five_lives() {
        assert_eq!(PlayerState::new(100).lives, 5);
    }

    #[test]
    fn full_lives_do_not_refill() {
        let mut state = PlayerState::new(0);
        let out = state.apply_refill(REFILL_INTERVAL_MS);
        assert_eq!(state.lives, 5);
        assert_eq!(out.ticks_applied, 0);
        assert_eq!(state.pending_ms, 0);
    }

    #[test]
    fn offline_under_interval_preserves_elapsed_time() {
        let mut state = PlayerState::new(0);
        state.lives = 3;
        state.apply_refill(10 * 60 * 1000);
        assert_eq!(state.lives, 3);
        assert_eq!(state.pending_ms, 10 * 60 * 1000);
    }

    #[test]
    fn offline_one_tick_refills_one_and_keeps_remainder() {
        let mut state = PlayerState::new(0);
        state.lives = 3;
        let out = state.apply_refill(35 * 60 * 1000);
        assert_eq!(state.lives, 4);
        assert_eq!(out.ticks_applied, 1);
        assert_eq!(state.pending_ms, 5 * 60 * 1000);
    }

    #[test]
    fn empty_lives_two_hours_refills_four() {
        let mut state = PlayerState::new(0);
        state.lives = 0;
        state.apply_refill(2 * 60 * 60 * 1000);
        assert_eq!(state.lives, 4);
        assert_eq!(state.pending_ms, 0);
    }

    #[test]
    fn refill_never_exceeds_max() {
        let mut state = PlayerState::new(0);
        state.lives = 4;
        state.apply_refill(2 * 60 * 60 * 1000);
        assert_eq!(state.lives, 5);
        assert_eq!(state.pending_ms, 0);
    }

    #[test]
    fn failure_costs_one_life_and_starts_timer_from_full() {
        let mut state = PlayerState::new(1_000);
        state.apply_level_outcome(LevelOutcome::Failure, 1_000);
        assert_eq!(state.lives, 4);
        assert_eq!(state.last_refill_at_ms, 1_000);
    }

    #[test]
    fn repeated_failure_does_not_reset_partial_timer() {
        let mut state = PlayerState::new(1_000);
        state.lives = 3;
        state.pending_ms = 10_000;
        state.apply_level_outcome(LevelOutcome::Failure, 99_000);
        assert_eq!(state.lives, 2);
        assert_eq!(state.pending_ms, 10_000);
        assert_eq!(state.last_refill_at_ms, 1_000);
    }

    #[test]
    fn success_preserves_lives_and_advances_progress() {
        let mut state = PlayerState::new(0);
        state.lives = 3;
        state.apply_level_outcome(LevelOutcome::Success, 1);
        assert_eq!(state.lives, 3);
        assert_eq!(state.level_progress, 1);
    }

    #[test]
    fn failure_at_zero_does_not_go_negative() {
        let mut state = PlayerState::new(0);
        state.lives = 0;
        state.apply_level_outcome(LevelOutcome::Failure, 10);
        assert_eq!(state.lives, 0);
    }

    #[test]
    fn clock_backwards_is_flagged_without_negative_elapsed() {
        let mut state = PlayerState::new(10_000);
        state.lives = 2;
        state.pending_ms = 2_000;
        let out = state.apply_refill(5_000);
        assert_eq!(out.clock_anomaly, ClockAnomaly::Backwards);
        assert_eq!(state.lives, 2);
        assert_eq!(state.pending_ms, 2_000);
        assert_eq!(state.last_refill_at_ms, 10_000);
    }

    #[test]
    fn implausible_forward_jump_is_flagged_and_rebased() {
        let mut state = PlayerState::new(1);
        state.lives = 2;
        let now = MAX_REASONABLE_OFFLINE_MS + 2;
        let out = state.apply_refill(now);
        assert_eq!(out.clock_anomaly, ClockAnomaly::TooFarForward);
        assert_eq!(state.lives, 2);
        assert_eq!(state.last_refill_at_ms, now);
        let next = state.apply_refill(now + REFILL_INTERVAL_MS);
        assert_eq!(next.clock_anomaly, ClockAnomaly::None);
        assert_eq!(state.lives, 3);
    }

    #[test]
    fn remaining_refill_time_is_bounded() {
        let mut state = PlayerState::new(0);
        state.lives = 4;
        state.pending_ms = 5 * 60 * 1000;
        assert_eq!(state.remaining_refill_ms(), 25 * 60 * 1000);
    }

    #[test]
    fn corrupt_out_of_range_lives_are_clamped() {
        let mut state = PlayerState::new(0);
        state.lives = 255;
        state.apply_refill(0);
        assert_eq!(state.lives, MAX_LIVES);
    }
}
