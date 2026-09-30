//! The countdown itself. Every read takes `now`, so the state is plain data
//! and tests can move time forward without sleeping.

use std::time::{Duration, Instant};

/// A countdown of a fixed length.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Timer {
    planned: Duration,
    since: Instant,
}

impl Timer {
    /// A running timer of `planned` length, started at `now`.
    pub fn start(planned: Duration, now: Instant) -> Self {
        Self {
            planned,
            since: now,
        }
    }

    /// The full length.
    pub fn planned(&self) -> Duration {
        self.planned
    }

    /// Time counted so far, never more than [`Timer::planned`].
    pub fn elapsed(&self, now: Instant) -> Duration {
        now.saturating_duration_since(self.since).min(self.planned)
    }

    /// Time left, zero once the timer is up.
    pub fn remaining(&self, now: Instant) -> Duration {
        self.planned.saturating_sub(self.elapsed(now))
    }
}

#[cfg(test)]
#[path = "timer_test.rs"]
mod tests;
