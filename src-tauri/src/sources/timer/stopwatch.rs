use std::time::Duration;

use crate::core::activity::{ActionSpec, Activity, CompactView};

use super::alert::Alert;
use super::chronometer::Chronometer;
use super::contract::{Control, Timer, TimerKind};
use super::format::clock_text;
use super::view::{build_activity, reset_action, status_label, toggle_action, LapView, TimerView};

const ACCENT: &str = "#38bdf8";

pub struct Stopwatch {
    clock: Chronometer,
    lap_totals: Vec<Duration>,
}

impl Stopwatch {
    pub fn start() -> Self {
        Self { clock: Chronometer::started(), lap_totals: Vec::new() }
    }

    fn record_lap(&mut self) {
        if self.clock.is_running() {
            self.lap_totals.push(self.clock.elapsed());
        }
    }

    fn lap_views(&self) -> Vec<LapView> {
        let mut previous = Duration::ZERO;
        let mut laps: Vec<LapView> = self
            .lap_totals
            .iter()
            .enumerate()
            .map(|(position, total)| {
                let split = total.saturating_sub(previous);
                previous = *total;
                LapView {
                    index: position + 1,
                    split_ms: split.as_millis() as u64,
                    total_ms: total.as_millis() as u64,
                }
            })
            .collect();
        laps.reverse();
        laps
    }

    fn actions(&self, running: bool) -> Vec<ActionSpec> {
        let mut actions = vec![toggle_action(running)];
        if running {
            actions.push(ActionSpec::button("lap", "Lap").icon("\u{2691}"));
        }
        actions.push(reset_action());
        actions
    }
}

impl Timer for Stopwatch {
    fn key(&self) -> &'static str {
        TimerKind::Stopwatch.key()
    }

    fn apply(&mut self, control: Control) {
        match control {
            Control::Pause => self.clock.pause(),
            Control::Resume => self.clock.resume(),
            Control::Toggle => self.clock.toggle(),
            Control::Lap => self.record_lap(),
            Control::Skip => {}
        }
    }

    fn take_finish_alert(&mut self) -> Option<Alert> {
        None
    }

    fn is_done(&self) -> bool {
        false
    }

    fn displayed_second(&self) -> u64 {
        self.clock.elapsed().as_secs()
    }

    fn activity(&self) -> Activity {
        let running = self.clock.is_running();
        let compact = CompactView::new("\u{23F1}", clock_text(self.displayed_second()))
            .subtitle(status_label("Stopwatch", running))
            .accent(ACCENT);
        let view = TimerView {
            mode: "stopwatch",
            label: "Stopwatch".into(),
            accent: ACCENT,
            running,
            elapsed_ms: self.clock.elapsed().as_millis() as u64,
            laps: self.lap_views(),
            ..TimerView::default()
        };
        build_activity(self.key(), compact, &view, self.actions(running))
    }
}
