//! Private host policy, deliberately not a runtime scheduler or playback API.
use std::time::Duration;

const PRESENTATION_CAP: Duration = Duration::from_millis(16);
const MIN_WAIT: Duration = Duration::from_millis(1);

pub struct Pacing {
    dirty: bool,
    last_presented: Option<Duration>,
    animation_due: Option<Duration>,
}
impl Default for Pacing {
    fn default() -> Self {
        Self {
            dirty: true,
            last_presented: None,
            animation_due: None,
        }
    }
}
impl Pacing {
    pub fn invalidate(&mut self, now: Duration, interval: Option<Duration>) {
        self.dirty = true;
        self.animation_due = interval.map(|interval| {
            self.last_presented
                .unwrap_or(now)
                .saturating_add(interval.max(PRESENTATION_CAP))
        });
    }
    pub fn presented(&mut self, now: Duration, interval: Option<Duration>) {
        self.dirty = false;
        self.last_presented = Some(now);
        // Restart from this presentation; never enqueue missed deadlines.
        self.animation_due =
            interval.map(|interval| now.saturating_add(interval.max(PRESENTATION_CAP)));
    }
    fn deadline(&self) -> Option<Duration> {
        let cap = self
            .last_presented
            .map(|last| last.saturating_add(PRESENTATION_CAP))
            .unwrap_or(Duration::ZERO);
        if self.dirty {
            Some(cap)
        } else {
            self.animation_due.map(|due| due.max(cap))
        }
    }
    pub fn should_draw(&self, now: Duration) -> bool {
        self.deadline().is_some_and(|due| due <= now)
    }
    pub fn wait(&self, now: Duration) -> Option<Duration> {
        self.deadline()
            .map(|due| due.saturating_sub(now).max(MIN_WAIT))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ms(value: u64) -> Duration {
        Duration::from_millis(value)
    }
    #[test]
    fn paused_and_static_hosts_have_no_animation_wakeups() {
        let mut pacing = Pacing::default();
        assert!(pacing.should_draw(ms(0)));
        pacing.presented(ms(0), None);
        for time in [16, 100, 10_000] {
            assert!(!pacing.should_draw(ms(time)));
            assert_eq!(pacing.wait(ms(time)), None);
        }
        pacing.invalidate(ms(10_001), None);
        assert!(pacing.should_draw(ms(10_001)));
        pacing.presented(ms(10_001), None);
        assert_eq!(pacing.wait(ms(10_002)), None);
    }
    #[test]
    fn event_storms_do_not_bypass_frame_cap_or_delay_input() {
        let mut pacing = Pacing::default();
        pacing.presented(ms(0), Some(ms(33)));
        let mut processed = 0;
        for time in 1..16 {
            processed += 1; // Events are handled immediately, between draws.
            pacing.invalidate(ms(time), Some(ms(33)));
            assert!(!pacing.should_draw(ms(time)));
            assert!(pacing.wait(ms(time)).unwrap() >= MIN_WAIT);
        }
        assert_eq!(processed, 15);
        assert!(pacing.should_draw(ms(16)));
    }
    #[test]
    fn long_stall_is_one_latest_frame_without_backlog() {
        let mut pacing = Pacing::default();
        pacing.presented(ms(0), Some(ms(33)));
        assert!(pacing.should_draw(ms(10_000)));
        pacing.presented(ms(10_000), Some(ms(33)));
        assert!(!pacing.should_draw(ms(10_000)));
        assert_eq!(pacing.wait(ms(10_000)), Some(ms(33)));
        pacing.invalidate(ms(10_001), None);
        assert!(!pacing.should_draw(ms(10_001)));
        pacing.presented(ms(10_016), None);
        assert_eq!(pacing.wait(ms(10_016)), None);
    }
}
