//! The countdown itself. Every read takes `now`, so the state is plain data
//! and tests can move time forward without sleeping.

use std::time::{Duration, Instant};

const MINUTE: Duration = Duration::from_secs(60);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Running { since: Instant },
    Paused,
}

/// A countdown that can pause, restart, and grow or shrink by a minute.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Timer {
    planned: Duration,
    banked: Duration,
    state: State,
}

impl Timer {
    /// A running timer of `planned` length, started at `now`.
    pub fn start(planned: Duration, now: Instant) -> Self {
        Self {
            planned,
            banked: Duration::ZERO,
            state: State::Running { since: now },
        }
    }

    /// The full length, including any minutes added or removed.
    pub fn planned(&self) -> Duration {
        self.planned
    }

    /// Time counted so far, never more than [`Timer::planned`].
    pub fn elapsed(&self, now: Instant) -> Duration {
        let running = match self.state {
            State::Running { since } => now.saturating_duration_since(since),
            State::Paused => Duration::ZERO,
        };
        (self.banked + running).min(self.planned)
    }

    /// Time left, zero once the timer is up.
    pub fn remaining(&self, now: Instant) -> Duration {
        self.planned.saturating_sub(self.elapsed(now))
    }

    /// Whether the countdown is frozen.
    pub fn is_paused(&self) -> bool {
        match self.state {
            State::Running { .. } => false,
            State::Paused => true,
        }
    }

    /// Pause a running timer, or resume a paused one.
    pub fn toggle(&mut self, now: Instant) {
        match self.state {
            State::Running { .. } => {
                self.banked = self.elapsed(now);
                self.state = State::Paused;
            }
            State::Paused => self.state = State::Running { since: now },
        }
    }

    /// Start the same length over from `now`, running.
    pub fn restart(&mut self, now: Instant) {
        self.banked = Duration::ZERO;
        self.state = State::Running { since: now };
    }

    /// One more minute on the clock.
    pub fn add_minute(&mut self) {
        self.planned += MINUTE;
    }

    /// One minute less, stopping at zero time left.
    pub fn remove_minute(&mut self, now: Instant) {
        self.planned = self.planned.saturating_sub(MINUTE).max(self.elapsed(now));
    }
}

#[cfg(test)]
#[path = "timer_test.rs"]
mod tests;
