use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use serde::Serialize;
use serde_json::Value;

use super::payload::summarize_tool_input;

const TAIL_BYTES: u64 = 768 * 1024;
const ITEM_LIMIT: usize = 60;
const USER_TEXT_LIMIT: usize = 2_000;
const ASSISTANT_TEXT_LIMIT: usize = 6_000;
const INTERRUPT_MARKER: &str = "[Request interrupted by user";
const TODO_TOOL: &str = "TodoWrite";
const HIDDEN_PREFIXES: [&str; 6] =
    ["<command-", "<local-command", "<system-reminder", "<bash-", "<user-prompt-submit-hook", "Caveat:"];

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum ChatItem {
    User { text: String },
    Assistant { text: String },
    Tool { id: String, name: String, summary: String, status: ToolStatus },
    Interrupted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolStatus {
    Running,
    Done,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Todo {
    content: String,
    status: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    pub items: Vec<ChatItem>,
    pub todos: Vec<Todo>,
}

pub fn read(path: &Path) -> Conversation {
    let Ok(length) = fs::metadata(path).map(|metadata| metadata.len()) else {
        return Conversation::default();
    };
    let start = length.saturating_sub(TAIL_BYTES);
    let Some(bytes) = read_range(path, start, length) else {
        return Conversation::default();
    };
    let text = String::from_utf8_lossy(&bytes);
    let complete = if start > 0 { text.split_once('\n').map_or("", |(_, rest)| rest) } else { &text };
    parse(complete)
}

/// Turns transcript lines into what the VS Code panel shows: prompts, replies, tool calls and the to-do list.
pub fn parse(lines: &str) -> Conversation {
    let mut conversation = Conversation::default();
    let mut tool_positions: HashMap<String, usize> = HashMap::new();
    for entry in lines.lines().filter_map(|line| serde_json::from_str::<Value>(line).ok()) {
        if entry.get("isSidechain").and_then(Value::as_bool) == Some(true) {
            continue;
        }
        match entry.get("type").and_then(Value::as_str) {
            Some("user") => absorb_user(&entry, &mut conversation, &tool_positions),
            Some("assistant") => absorb_assistant(&entry, &mut conversation, &mut tool_positions),
            _ => {}
        }
    }
    let overflow = conversation.items.len().saturating_sub(ITEM_LIMIT);
    conversation.items.drain(..overflow);
    conversation
}

fn absorb_user(entry: &Value, conversation: &mut Conversation, tools: &HashMap<String, usize>) {
    if entry.get("isMeta").and_then(Value::as_bool) == Some(true) {
        return;
    }
    match entry.pointer("/message/content") {
        Some(Value::String(text)) => push_prompt(conversation, text),
        Some(Value::Array(blocks)) => {
            for block in blocks {
                match block.get("type").and_then(Value::as_str) {
                    Some("text") => push_prompt(conversation, block.get("text").and_then(Value::as_str).unwrap_or("")),
                    Some("tool_result") => finish_tool(conversation, tools, block),
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

fn push_prompt(conversation: &mut Conversation, text: &str) {
    let trimmed = text.trim();
    if trimmed.is_empty() || HIDDEN_PREFIXES.iter().any(|prefix| trimmed.starts_with(prefix)) {
        return;
    }
    if trimmed.starts_with(INTERRUPT_MARKER) {
        conversation.items.push(ChatItem::Interrupted);
        return;
    }
    conversation.items.push(ChatItem::User { text: limit(trimmed, USER_TEXT_LIMIT) });
}

fn finish_tool(conversation: &mut Conversation, tools: &HashMap<String, usize>, block: &Value) {
    let Some(position) = block.get("tool_use_id").and_then(Value::as_str).and_then(|id| tools.get(id)) else {
        return;
    };
    let failed = block.get("is_error").and_then(Value::as_bool) == Some(true);
    if let Some(ChatItem::Tool { status, .. }) = conversation.items.get_mut(*position) {
        *status = if failed { ToolStatus::Error } else { ToolStatus::Done };
    }
}

fn absorb_assistant(entry: &Value, conversation: &mut Conversation, tools: &mut HashMap<String, usize>) {
    let Some(Value::Array(blocks)) = entry.pointer("/message/content") else {
        return;
    };
    for block in blocks {
        match block.get("type").and_then(Value::as_str) {
            Some("text") => {
                let text = block.get("text").and_then(Value::as_str).unwrap_or("").trim();
                if !text.is_empty() {
                    conversation.items.push(ChatItem::Assistant { text: limit(text, ASSISTANT_TEXT_LIMIT) });
                }
            }
            Some("tool_use") => push_tool(block, conversation, tools),
            _ => {}
        }
    }
}

fn push_tool(block: &Value, conversation: &mut Conversation, tools: &mut HashMap<String, usize>) {
    let name = block.get("name").and_then(Value::as_str).unwrap_or("Tool").to_string();
    let input = block.get("input").unwrap_or(&Value::Null);
    if name == TODO_TOOL {
        conversation.todos = todos_from(input);
    }
    let id = block.get("id").and_then(Value::as_str).unwrap_or_default().to_string();
    tools.insert(id.clone(), conversation.items.len());
    conversation.items.push(ChatItem::Tool { id, name, summary: summarize_tool_input(input), status: ToolStatus::Running });
}

fn todos_from(input: &Value) -> Vec<Todo> {
    let Some(Value::Array(todos)) = input.get("todos") else {
        return Vec::new();
    };
    todos
        .iter()
        .filter_map(|todo| {
            Some(Todo {
                content: todo.get("content")?.as_str()?.to_string(),
                status: todo.get("status").and_then(Value::as_str).unwrap_or("pending").to_string(),
            })
        })
        .collect()
}

fn limit(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let kept: String = text.chars().take(max_chars).collect();
    format!("{kept}…")
}

fn read_range(path: &Path, start: u64, end: u64) -> Option<Vec<u8>> {
    let mut file = File::open(path).ok()?;
    file.seek(SeekFrom::Start(start)).ok()?;
    let mut bytes = Vec::new();
    file.take(end - start).read_to_end(&mut bytes).ok()?;
    Some(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(entries: &[&str]) -> String {
        entries.join("\n")
    }

    const PROMPT: &str = r#"{"type":"user","message":{"role":"user","content":"fix the crash"}}"#;
    const REPLY: &str = r#"{"type":"assistant","message":{"content":[{"type":"text","text":"Looking at it now."}]}}"#;
    const TOOL_USE: &str = r#"{"type":"assistant","message":{"content":[{"type":"tool_use","id":"t1","name":"Bash","input":{"command":"cargo test"}}]}}"#;
    const TOOL_OK: &str = r#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"t1","content":"ok"}]}}"#;
    const TOOL_FAIL: &str = r#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"t1","content":"boom","is_error":true}]}}"#;

    #[test]
    fn reads_prompts_replies_and_tool_calls_in_order() {
        let conversation = parse(&lines(&[PROMPT, REPLY, TOOL_USE]));
        assert_eq!(
            conversation.items,
            vec![
                ChatItem::User { text: "fix the crash".into() },
                ChatItem::Assistant { text: "Looking at it now.".into() },
                ChatItem::Tool { id: "t1".into(), name: "Bash".into(), summary: "cargo test".into(), status: ToolStatus::Running },
            ]
        );
    }

    #[test]
    fn tool_results_mark_the_call_done_or_failed() {
        let done = parse(&lines(&[TOOL_USE, TOOL_OK]));
        assert!(matches!(done.items[0], ChatItem::Tool { status: ToolStatus::Done, .. }));
        let failed = parse(&lines(&[TOOL_USE, TOOL_FAIL]));
        assert!(matches!(failed.items[0], ChatItem::Tool { status: ToolStatus::Error, .. }));
    }

    #[test]
    fn hides_meta_messages_subagents_and_command_wrappers() {
        let meta = r#"{"type":"user","isMeta":true,"message":{"content":"Caveat: generated"}}"#;
        let side = r#"{"type":"assistant","isSidechain":true,"message":{"content":[{"type":"text","text":"subagent"}]}}"#;
        let command = r#"{"type":"user","message":{"content":"<command-name>/clear</command-name>"}}"#;
        assert!(parse(&lines(&[meta, side, command])).items.is_empty());
    }

    #[test]
    fn shows_interrupts_and_keeps_the_latest_todos() {
        let interrupt = r#"{"type":"user","message":{"content":[{"type":"text","text":"[Request interrupted by user]"}]}}"#;
        let todos = r#"{"type":"assistant","message":{"content":[{"type":"tool_use","id":"t2","name":"TodoWrite","input":{"todos":[{"content":"Write tests","status":"in_progress","activeForm":"Writing tests"}]}}]}}"#;
        let conversation = parse(&lines(&[todos, interrupt]));
        assert_eq!(conversation.items.last(), Some(&ChatItem::Interrupted));
        assert_eq!(conversation.todos, vec![Todo { content: "Write tests".into(), status: "in_progress".into() }]);
    }

    #[test]
    fn keeps_only_the_most_recent_items() {
        let many: Vec<String> = (0..80).map(|index| format!(r#"{{"type":"user","message":{{"content":"prompt {index}"}}}}"#)).collect();
        let conversation = parse(&many.join("\n"));
        assert_eq!(conversation.items.len(), ITEM_LIMIT);
        assert_eq!(conversation.items[0], ChatItem::User { text: "prompt 20".into() });
    }
}
