//! Inactivity tracking for the automatic lock.

use std::time::{Duration, Instant};

/// The vault locks after this much time without user activity.
pub const IDLE_TIMEOUT: Duration = Duration::from_secs(5 * 60);
/// How often the background watcher checks for inactivity.
pub const CHECK_INTERVAL: Duration = Duration::from_secs(5);

/// Remembers the time of the last user activity.
///
/// Time is passed in by the caller, so the logic is tested without waiting.
#[derive(Debug, Clone, Copy)]
pub struct ActivityTracker {
    last_activity: Instant,
    timeout: Duration,
}

impl ActivityTracker {
    /// A tracker whose last activity is `now`.
    pub fn new(now: Instant, timeout: Duration) -> ActivityTracker {
        ActivityTracker {
            last_activity: now,
            timeout,
        }
    }

    /// Records activity at `now`. Times earlier than the last activity are
    /// ignored, so the deadline never moves backwards.
    pub fn touch(&mut self, now: Instant) {
        if now > self.last_activity {
            self.last_activity = now;
        }
    }

    /// Whether `now` is at or past the inactivity deadline.
    pub fn is_idle(&self, now: Instant) -> bool {
        now.saturating_duration_since(self.last_activity) >= self.timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_not_idle_before_the_timeout() {
        let start = Instant::now();
        let tracker = ActivityTracker::new(start, IDLE_TIMEOUT);
        assert!(!tracker.is_idle(start));
        assert!(!tracker.is_idle(start + IDLE_TIMEOUT - Duration::from_millis(1)));
    }

    #[test]
    fn is_idle_at_and_after_the_timeout() {
        let start = Instant::now();
        let tracker = ActivityTracker::new(start, IDLE_TIMEOUT);
        assert!(tracker.is_idle(start + IDLE_TIMEOUT));
        assert!(tracker.is_idle(start + IDLE_TIMEOUT * 2));
    }

    #[test]
    fn activity_moves_the_deadline() {
        let start = Instant::now();
        let mut tracker = ActivityTracker::new(start, IDLE_TIMEOUT);
        tracker.touch(start + Duration::from_secs(240));
        assert!(!tracker.is_idle(start + IDLE_TIMEOUT));
        assert!(tracker.is_idle(start + Duration::from_secs(240) + IDLE_TIMEOUT));
    }

    #[test]
    fn earlier_times_do_not_move_the_deadline_back() {
        let start = Instant::now();
        let mut tracker = ActivityTracker::new(start + Duration::from_secs(60), IDLE_TIMEOUT);
        tracker.touch(start);
        assert!(!tracker.is_idle(start + IDLE_TIMEOUT));
    }

    #[test]
    fn a_time_before_the_last_activity_is_not_idle() {
        let start = Instant::now();
        let tracker = ActivityTracker::new(start + Duration::from_secs(10), IDLE_TIMEOUT);
        assert!(!tracker.is_idle(start));
    }
}
