use std::time::Duration;

use super::payload::{summarize_tool_input, HookPayload};

pub const STALE_AFTER: Duration = Duration::from_secs(120);

#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    Started,
    Thinking,
    RunningTool { tool: String, detail: String },
    WaitingForInput { message: String },
    AwaitingFollowUp { message: String },
    Done,
    Interrupted,
    Idle,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Transition {
    Show(SessionState),
    Remove,
    Ignore,
}

impl SessionState {
    pub fn is_working(&self) -> bool {
        matches!(self, Self::Thinking | Self::RunningTool { .. })
    }

    pub fn can_be_interrupted(&self) -> bool {
        self.is_working() || matches!(self, Self::WaitingForInput { .. })
    }

    fn is_finished(&self) -> bool {
        matches!(self, Self::Done | Self::Interrupted | Self::Idle | Self::AwaitingFollowUp { .. })
    }

    pub fn goes_stale_after(&self, silence: Duration) -> bool {
        self.is_working() && silence >= STALE_AFTER
    }
}

pub fn transition(previous: Option<&SessionState>, event: &str, payload: &HookPayload) -> Transition {
    match event {
        "SessionEnd" => Transition::Remove,
        "Notification" => Transition::Show(notification_state(previous, payload)),
        _ => state_after(event, payload).map_or(Transition::Ignore, Transition::Show),
    }
}

fn notification_state(previous: Option<&SessionState>, payload: &HookPayload) -> SessionState {
    let message = payload.message().unwrap_or("Waiting for your input").to_owned();
    if previous.is_some_and(SessionState::is_finished) {
        SessionState::AwaitingFollowUp { message }
    } else {
        SessionState::WaitingForInput { message }
    }
}

fn state_after(event: &str, payload: &HookPayload) -> Option<SessionState> {
    match event {
        "SessionStart" => Some(SessionState::Started),
        "UserPromptSubmit" | "PostToolUse" => Some(SessionState::Thinking),
        "PreToolUse" => Some(SessionState::RunningTool {
            tool: payload.tool_name().to_owned(),
            detail: summarize_tool_input(payload.tool_input()),
        }),
        "Stop" => Some(SessionState::Done),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload(json: &str) -> HookPayload {
        HookPayload::parse(json.as_bytes())
    }

    fn shown(previous: Option<&SessionState>, event: &str, json: &str) -> Transition {
        transition(previous, event, &payload(json))
    }

    #[test]
    fn pre_tool_use_reports_tool_and_argument() {
        let expected = SessionState::RunningTool { tool: "Bash".into(), detail: "cargo build".into() };
        let body = r#"{"tool_name":"Bash","tool_input":{"command":"cargo build"}}"#;
        assert_eq!(shown(None, "PreToolUse", body), Transition::Show(expected));
    }

    #[test]
    fn post_tool_use_returns_to_thinking() {
        assert_eq!(shown(None, "PostToolUse", "{}"), Transition::Show(SessionState::Thinking));
    }

    #[test]
    fn notification_during_work_waits_with_its_message() {
        let expected = SessionState::WaitingForInput { message: "Claude needs you".into() };
        let body = r#"{"message":"Claude needs you"}"#;
        assert_eq!(shown(Some(&SessionState::Thinking), "Notification", body), Transition::Show(expected.clone()));
        assert_eq!(shown(None, "Notification", body), Transition::Show(expected));
    }

    #[test]
    fn notification_after_a_finished_turn_never_revives_work() {
        let expected = SessionState::AwaitingFollowUp { message: "Claude is waiting".into() };
        let body = r#"{"message":"Claude is waiting"}"#;
        for finished in [SessionState::Done, SessionState::Interrupted, SessionState::Idle] {
            assert_eq!(shown(Some(&finished), "Notification", body), Transition::Show(expected.clone()));
        }
    }

    #[test]
    fn session_end_removes_the_card() {
        assert_eq!(shown(Some(&SessionState::Thinking), "SessionEnd", "{}"), Transition::Remove);
    }

    #[test]
    fn stop_finishes_and_unknown_events_are_ignored() {
        assert_eq!(shown(None, "Stop", "{}"), Transition::Show(SessionState::Done));
        assert_eq!(shown(None, "PermissionRequest", "{}"), Transition::Ignore);
    }

    #[test]
    fn only_working_states_go_stale() {
        let long_silence = STALE_AFTER + Duration::from_secs(1);
        let tool = SessionState::RunningTool { tool: "Bash".into(), detail: String::new() };
        assert!(SessionState::Thinking.goes_stale_after(long_silence));
        assert!(tool.goes_stale_after(long_silence));
        assert!(!SessionState::Thinking.goes_stale_after(STALE_AFTER - Duration::from_secs(1)));
        assert!(!SessionState::Done.goes_stale_after(long_silence));
        assert!(!SessionState::WaitingForInput { message: String::new() }.goes_stale_after(long_silence));
    }

    #[test]
    fn waiting_for_input_can_still_be_interrupted() {
        assert!(SessionState::WaitingForInput { message: String::new() }.can_be_interrupted());
        assert!(!SessionState::Done.can_be_interrupted());
    }
}
