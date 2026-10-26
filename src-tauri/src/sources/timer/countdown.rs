use std::time::Duration;

use crate::core::activity::{Activity, CompactView};

use super::alert::Alert;
use super::chronometer::Chronometer;
use super::contract::{Control, Timer, TimerKind};
use super::format::{clock_text, whole_seconds_rounded_up};
use super::view::{build_activity, reset_action, status_label, toggle_action, TimerView};

const ACCENT: &str = "#f59e0b";
const ALERT_TTL_MS: u64 = 20_000;

pub struct Countdown {
    total: Duration,
    clock: Chronometer,
    alerted: bool,
}

impl Countdown {
    pub fn start(total: Duration) -> Self {
        Self { total, clock: Chronometer::started(), alerted: false }
    }

    fn remaining(&self) -> Duration {
        self.total.saturating_sub(self.clock.elapsed())
    }

    fn has_elapsed(&self) -> bool {
        self.remaining().is_zero()
    }
}

impl Timer for Countdown {
    fn key(&self) -> &'static str {
        TimerKind::Countdown.key()
    }

    fn apply(&mut self, control: Control) {
        match control {
            Control::Pause => self.clock.pause(),
            Control::Resume => self.clock.resume(),
            Control::Toggle => self.clock.toggle(),
            Control::Lap | Control::Skip => {}
        }
    }

    fn take_finish_alert(&mut self) -> Option<Alert> {
        if !self.has_elapsed() || self.alerted {
            return None;
        }
        self.alerted = true;
        Some(Alert {
            key: "countdown-done",
            icon: "\u{23F0}",
            title: "Time's up".into(),
            message: format!("{} countdown finished", clock_text(self.total.as_secs())),
            accent: ACCENT,
            ttl_ms: ALERT_TTL_MS,
        })
    }

    fn is_done(&self) -> bool {
        self.alerted
    }

    fn displayed_second(&self) -> u64 {
        whole_seconds_rounded_up(self.remaining())
    }

    fn activity(&self) -> Activity {
        let running = self.clock.is_running();
        let compact = CompactView::new("\u{23F3}", clock_text(self.displayed_second()))
            .subtitle(status_label("Timer", running))
            .accent(ACCENT);
        let view = TimerView {
            mode: "countdown",
            label: "Timer".into(),
            accent: ACCENT,
            running,
            elapsed_ms: self.clock.elapsed().as_millis() as u64,
            total_ms: Some(self.total.as_millis() as u64),
            ..TimerView::default()
        };
        build_activity(self.key(), compact, &view, vec![toggle_action(running), reset_action()])
    }
}
