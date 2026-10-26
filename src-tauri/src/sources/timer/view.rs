use serde::Serialize;

use crate::core::activity::{ActionSpec, ActionStyle, Activity, ActivityKind, CompactView};

pub const SOURCE_NAME: &str = "timer";

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LapView {
    pub index: usize,
    pub split_ms: u64,
    pub total_ms: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionView {
    pub completed: u32,
    pub of: u32,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TimerView {
    pub mode: &'static str,
    pub label: String,
    pub accent: &'static str,
    pub running: bool,
    pub elapsed_ms: u64,
    pub total_ms: Option<u64>,
    pub laps: Vec<LapView>,
    pub session: Option<SessionView>,
}

pub fn build_activity(
    key: &str,
    compact: CompactView,
    view: &TimerView,
    actions: Vec<ActionSpec>,
) -> Activity {
    let expanded = serde_json::to_value(view).unwrap_or_default();
    Activity::new(SOURCE_NAME, key, ActivityKind::Timer, compact)
        .expanded(expanded)
        .actions(actions)
}

pub fn toggle_action(running: bool) -> ActionSpec {
    if running {
        ActionSpec::button("toggle", "Pause").icon("\u{23F8}")
    } else {
        ActionSpec::button("toggle", "Resume").icon("\u{25B6}").style(ActionStyle::Primary)
    }
}

pub fn reset_action() -> ActionSpec {
    ActionSpec::button("reset", "Reset").icon("\u{21BA}").style(ActionStyle::Danger)
}

pub fn status_label(base: &str, running: bool) -> String {
    if running {
        base.to_string()
    } else {
        format!("{base} \u{00B7} paused")
    }
}
