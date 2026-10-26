use serde_json::json;

use super::payload::{preview_tool_input, summarize_tool_input, HookPayload};
use super::sessions::SessionState;
use crate::core::activity::{ActionSpec, ActionStyle, Activity, ActivityKind, CompactView, Priority};
use crate::core::source::SourceContext;

const CLAUDE_ACCENT: &str = "#d97757";
const DONE_ACCENT: &str = "#22c55e";
const ACTIVE_SESSION_TTL_MS: u64 = 15 * 60 * 1000;
const DONE_TTL_MS: u64 = 8_000;

struct StatusLine {
    label: &'static str,
    text: String,
}

pub fn session_activity(
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
        .ttl_ms(ttl_for(state));
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
        SessionState::Done => ("done", "Done".to_owned()),
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
        _ => CLAUDE_ACCENT,
    }
}

fn ttl_for(state: &SessionState) -> u64 {
    match state {
        SessionState::Done => DONE_TTL_MS,
        _ => ACTIVE_SESSION_TTL_MS,
    }
}

fn prioritized(activity: Activity, state: &SessionState) -> Activity {
    match state {
        SessionState::WaitingForInput { .. } => activity.priority(Priority::High).alert(true),
        SessionState::Done => activity.priority(Priority::High),
        _ => activity,
    }
}
