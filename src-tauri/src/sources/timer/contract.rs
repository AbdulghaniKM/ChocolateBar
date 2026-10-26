use serde::Deserialize;

use crate::core::activity::Activity;

use super::alert::Alert;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimerKind {
    Countdown,
    Stopwatch,
    Pomodoro,
}

impl TimerKind {
    pub fn key(self) -> &'static str {
        match self {
            Self::Countdown => "countdown",
            Self::Stopwatch => "stopwatch",
            Self::Pomodoro => "pomodoro",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Control {
    Pause,
    Resume,
    Toggle,
    Lap,
    Skip,
}

impl Control {
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "pause" => Some(Self::Pause),
            "resume" | "start" => Some(Self::Resume),
            "toggle" => Some(Self::Toggle),
            "lap" => Some(Self::Lap),
            "skip" => Some(Self::Skip),
            _ => None,
        }
    }
}

pub trait Timer: Send {
    fn key(&self) -> &'static str;
    fn apply(&mut self, control: Control);
    fn take_finish_alert(&mut self) -> Option<Alert>;
    fn is_done(&self) -> bool;
    fn displayed_second(&self) -> u64;
    fn activity(&self) -> Activity;
}
