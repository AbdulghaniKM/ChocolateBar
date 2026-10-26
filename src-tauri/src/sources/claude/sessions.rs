use super::payload::{summarize_tool_input, HookPayload};

#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    Started,
    Thinking,
    RunningTool { tool: String, detail: String },
    WaitingForInput { message: String },
    Done,
}

pub fn state_after(event: &str, payload: &HookPayload) -> Option<SessionState> {
    match event {
        "SessionStart" => Some(SessionState::Started),
        "UserPromptSubmit" | "PostToolUse" => Some(SessionState::Thinking),
        "PreToolUse" => Some(SessionState::RunningTool {
            tool: payload.tool_name().to_owned(),
            detail: summarize_tool_input(payload.tool_input()),
        }),
        "Notification" => Some(SessionState::WaitingForInput {
            message: payload.message().unwrap_or("Waiting for your input").to_owned(),
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

    #[test]
    fn pre_tool_use_reports_tool_and_argument() {
        let body = payload(r#"{"tool_name":"Bash","tool_input":{"command":"cargo build"}}"#);
        let expected = SessionState::RunningTool { tool: "Bash".into(), detail: "cargo build".into() };
        assert_eq!(state_after("PreToolUse", &body), Some(expected));
    }

    #[test]
    fn post_tool_use_returns_to_thinking() {
        assert_eq!(state_after("PostToolUse", &payload("{}")), Some(SessionState::Thinking));
    }

    #[test]
    fn notification_waits_with_its_message() {
        let expected = SessionState::WaitingForInput { message: "Claude needs you".into() };
        assert_eq!(state_after("Notification", &payload(r#"{"message":"Claude needs you"}"#)), Some(expected));
    }

    #[test]
    fn permission_request_does_not_change_session_state() {
        assert_eq!(state_after("PermissionRequest", &payload("{}")), None);
    }
}
