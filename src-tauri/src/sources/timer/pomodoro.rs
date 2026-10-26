use std::time::Duration;

use crate::core::activity::{ActionSpec, Activity, CompactView};
use crate::core::settings::TimerSettings;

use super::alert::Alert;
use super::chronometer::Chronometer;
use super::contract::{Control, Timer, TimerKind};
use super::format::{clock_text, whole_seconds_rounded_up};
use super::view::{
    build_activity, reset_action, status_label, toggle_action, SessionView, TimerView,
};

const WORK_SESSIONS_PER_CYCLE: u32 = 4;
const ALERT_TTL_MS: u64 = 8_000;
const SECONDS_PER_MINUTE: u64 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Work,
    ShortBreak,
    LongBreak,
}

impl Phase {
    fn label(self) -> &'static str {
        match self {
            Self::Work => "Focus",
            Self::ShortBreak => "Short break",
            Self::LongBreak => "Long break",
        }
    }

    fn accent(self) -> &'static str {
        match self {
            Self::Work => "#f59e0b",
            Self::ShortBreak => "#22c55e",
            Self::LongBreak => "#38bdf8",
        }
    }

    fn icon(self) -> &'static str {
        if self == Self::Work {
            "\u{1F345}"
        } else {
            "\u{2615}"
        }
    }
}

pub struct Pomodoro {
    plan: TimerSettings,
    phase: Phase,
    completed_work_sessions: u32,
    clock: Chronometer,
}

impl Pomodoro {
    pub fn start(plan: TimerSettings) -> Self {
        Self {
            plan,
            phase: Phase::Work,
            completed_work_sessions: 0,
            clock: Chronometer::started(),
        }
    }

    fn phase_length(&self) -> Duration {
        let minutes = match self.phase {
            Phase::Work => self.plan.work_min,
            Phase::ShortBreak => self.plan.short_min,
            Phase::LongBreak => self.plan.long_min,
        };
        Duration::from_secs(u64::from(minutes.max(1)) * SECONDS_PER_MINUTE)
    }

    fn remaining(&self) -> Duration {
        self.phase_length().saturating_sub(self.clock.elapsed())
    }

    fn advance_phase(&mut self) {
        self.phase = match self.phase {
            Phase::Work => self.phase_after_work(),
            Phase::ShortBreak | Phase::LongBreak => Phase::Work,
        };
        self.clock.restart();
    }

    fn phase_after_work(&mut self) -> Phase {
        self.completed_work_sessions += 1;
        if self.completed_work_sessions < WORK_SESSIONS_PER_CYCLE {
            return Phase::ShortBreak;
        }
        self.completed_work_sessions = 0;
        Phase::LongBreak
    }

    fn sessions_shown_as_completed(&self) -> u32 {
        if self.phase == Phase::LongBreak {
            WORK_SESSIONS_PER_CYCLE
        } else {
            self.completed_work_sessions
        }
    }

    fn alert_for_end_of(&self, ended: Phase) -> Alert {
        let upcoming = self.phase;
        Alert {
            key: "pomodoro-alert",
            icon: ended.icon(),
            title: format!("{} complete", ended.label()),
            message: format!("Up next: {}", upcoming.label().to_lowercase()),
            accent: upcoming.accent(),
            ttl_ms: ALERT_TTL_MS,
        }
    }
}

impl Timer for Pomodoro {
    fn key(&self) -> &'static str {
        TimerKind::Pomodoro.key()
    }

    fn apply(&mut self, control: Control) {
        match control {
            Control::Pause => self.clock.pause(),
            Control::Resume => self.clock.resume(),
            Control::Toggle => self.clock.toggle(),
            Control::Skip => self.advance_phase(),
            Control::Lap => {}
        }
    }

    fn take_finish_alert(&mut self) -> Option<Alert> {
        if !self.remaining().is_zero() {
            return None;
        }
        let ended = self.phase;
        self.advance_phase();
        Some(self.alert_for_end_of(ended))
    }

    fn is_done(&self) -> bool {
        false
    }

    fn displayed_second(&self) -> u64 {
        whole_seconds_rounded_up(self.remaining())
    }

    fn activity(&self) -> Activity {
        let running = self.clock.is_running();
        let accent = self.phase.accent();
        let compact = CompactView::new(self.phase.icon(), clock_text(self.displayed_second()))
            .subtitle(status_label(self.phase.label(), running))
            .accent(accent);
        let view = TimerView {
            mode: "pomodoro",
            label: self.phase.label().into(),
            accent,
            running,
            elapsed_ms: self.clock.elapsed().as_millis() as u64,
            total_ms: Some(self.phase_length().as_millis() as u64),
            session: Some(SessionView {
                completed: self.sessions_shown_as_completed(),
                of: WORK_SESSIONS_PER_CYCLE,
            }),
            ..TimerView::default()
        };
        let skip = ActionSpec::button("skip", "Skip").icon("\u{23ED}");
        let actions = vec![toggle_action(running), skip, reset_action()];
        build_activity(self.key(), compact, &view, actions)
    }
}
