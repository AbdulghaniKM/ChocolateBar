use serde_json::json;

use super::payload::{preview_tool_input, summarize_tool_input, HookPayload};
use super::sessions::SessionState;
use super::tracker::CardChange;
use crate::core::activity::{ActionSpec, ActionStyle, Activity, ActivityKind, CompactView, Priority};
use crate::core::source::SourceContext;

const CLAUDE_ACCENT: &str = "#d97757";
const DONE_ACCENT: &str = "#22c55e";
const ACTIVE_SESSION_TTL_MS: u64 = 12 * 60 * 60 * 1000;
const DONE_TTL_MS: u64 = 8_000;
const FOLLOW_UP_TTL_MS: u64 = 15_000;
const INTERRUPTED_ACCENT: &str = "#f59e0b";
pub const DISMISS_ACTION: &str = "dismiss";

struct StatusLine {
    label: &'static str,
    text: String,
}

pub fn apply_change(ctx: &SourceContext, change: &CardChange) {
    match change {
        CardChange::Show { session_id, project, state } => {
            ctx.upsert(session_activity(ctx, session_id, project.as_deref(), state));
        }
        CardChange::Remove { session_id } => ctx.remove(session_id),
    }
}

fn session_activity(
    ctx: &SourceContext,
    session_id: &str,
    project: Option<&str>,
    state: &SessionState,
) -> Activity {
    let status = status_line(state);
    let title = project.unwrap_or("Claude Code");
    let compact = CompactView::new("✦", title)
        .subtitle(status.text.clone())
        .accent(accent_for(state));
    let expanded = json!({
        "view": "session",
        "state": status.label,
        "project": project,
        "statusLine": status.text,
        "tool": tool_of(state),
    });
    let activity = ctx
        .activity(session_id, ActivityKind::Claude, compact)
        .expanded(expanded)
        .ttl_ms(ttl_for(state))
        .action(ActionSpec::button(DISMISS_ACTION, "Dismiss"));
    prioritized(activity, state)
}

pub fn permission_activity(ctx: &SourceContext, request_key: &str, payload: &HookPayload) -> Activity {
    let tool = payload.tool_name();
    let compact = CompactView::new("🔐", format!("Allow {tool}?"))
        .subtitle(summarize_tool_input(payload.tool_input()))
        .accent(CLAUDE_ACCENT)
        .badge("Claude");
    let expanded = json!({
        "view": "permission",
        "toolName": tool,
        "inputPreview": preview_tool_input(payload.tool_input()),
        "project": payload.project_name(),
    });
    ctx.activity(request_key, ActivityKind::Claude, compact)
        .priority(Priority::Critical)
        .alert(true)
        .expanded(expanded)
        .action(ActionSpec::button("allow", "Allow").icon("✓").style(ActionStyle::Primary))
        .action(ActionSpec::button("deny", "Deny").icon("✕").style(ActionStyle::Danger))
}

fn status_line(state: &SessionState) -> StatusLine {
    let (label, text) = match state {
        SessionState::Started => ("started", "Session started".to_owned()),
        SessionState::Thinking => ("thinking", "Thinking…".to_owned()),
        SessionState::RunningTool { tool, detail } if detail.is_empty() => {
            ("tool", format!("Running {tool}"))
        }
        SessionState::RunningTool { tool, detail } => ("tool", format!("{tool} · {detail}")),
        SessionState::WaitingForInput { message } => ("waiting", message.clone()),
        SessionState::AwaitingFollowUp { message } => ("waiting", message.clone()),
        SessionState::Done => ("done", "Done".to_owned()),
        SessionState::Interrupted => ("interrupted", "Interrupted".to_owned()),
        SessionState::Idle => ("idle", "Idle".to_owned()),
    };
    StatusLine { label, text }
}

fn tool_of(state: &SessionState) -> Option<&str> {
    match state {
        SessionState::RunningTool { tool, .. } => Some(tool),
        _ => None,
    }
}

fn accent_for(state: &SessionState) -> &'static str {
    match state {
        SessionState::Done => DONE_ACCENT,
        SessionState::Interrupted => INTERRUPTED_ACCENT,
        _ => CLAUDE_ACCENT,
    }
}

fn ttl_for(state: &SessionState) -> u64 {
    match state {
        SessionState::Done | SessionState::Interrupted | SessionState::Idle => DONE_TTL_MS,
        SessionState::AwaitingFollowUp { .. } => FOLLOW_UP_TTL_MS,
        _ => ACTIVE_SESSION_TTL_MS,
    }
}

fn prioritized(activity: Activity, state: &SessionState) -> Activity {
    match state {
        SessionState::WaitingForInput { .. } => activity.priority(Priority::High).alert(true),
        SessionState::Done | SessionState::Interrupted => activity.priority(Priority::High),
        _ => activity,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_flight_states_outlive_a_long_tool_run() {
        let one_hour_ms = 60 * 60 * 1000;
        let waiting = SessionState::WaitingForInput { message: String::new() };
        for state in [SessionState::Started, SessionState::Thinking, waiting] {
            assert!(ttl_for(&state) > one_hour_ms);
        }
    }

    #[test]
    fn finished_cards_expire_quickly() {
        let follow_up = SessionState::AwaitingFollowUp { message: String::new() };
        let finished = [SessionState::Done, SessionState::Interrupted, SessionState::Idle, follow_up];
        for state in finished {
            assert!(ttl_for(&state) < 60_000);
        }
    }

    #[test]
    fn interrupted_and_idle_are_labelled_distinctly_from_thinking() {
        assert_eq!(status_line(&SessionState::Interrupted).label, "interrupted");
        assert_eq!(status_line(&SessionState::Idle).label, "idle");
        assert_ne!(status_line(&SessionState::Idle).label, status_line(&SessionState::Thinking).label);
    }
}
