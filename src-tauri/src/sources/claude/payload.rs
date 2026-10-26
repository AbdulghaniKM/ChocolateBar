use std::path::Path;

use serde_json::Value;

const SUMMARY_LIMIT: usize = 64;
const PREVIEW_LIMIT: usize = 600;
const SUMMARY_KEYS: [&str; 8] =
    ["command", "file_path", "path", "pattern", "url", "query", "description", "prompt"];
const PATH_KEYS: [&str; 2] = ["file_path", "path"];

pub struct HookPayload(Value);

impl HookPayload {
    pub fn parse(body: &[u8]) -> Self {
        Self(serde_json::from_slice(body).unwrap_or(Value::Null))
    }

    fn text(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(Value::as_str).filter(|text| !text.is_empty())
    }

    pub fn session_id(&self) -> &str {
        self.text("session_id").unwrap_or("default")
    }

    pub fn tool_name(&self) -> &str {
        self.text("tool_name").unwrap_or("tool")
    }

    pub fn tool_input(&self) -> &Value {
        self.0.get("tool_input").unwrap_or(&Value::Null)
    }

    pub fn message(&self) -> Option<&str> {
        self.text("message")
    }

    pub fn project_name(&self) -> Option<String> {
        let working_directory = self.text("cwd")?;
        let name = Path::new(working_directory).file_name()?;
        Some(name.to_string_lossy().into_owned())
    }
}

pub fn summarize_tool_input(input: &Value) -> String {
    let summary = SUMMARY_KEYS
        .iter()
        .find_map(|key| text_for_key(input, key))
        .unwrap_or_default();
    truncate(&summary, SUMMARY_LIMIT)
}

pub fn preview_tool_input(input: &Value) -> String {
    let preview = match input.get("command").and_then(Value::as_str) {
        Some(command) => command.to_owned(),
        None => serde_json::to_string_pretty(input).unwrap_or_default(),
    };
    truncate(&preview, PREVIEW_LIMIT)
}

fn text_for_key(input: &Value, key: &str) -> Option<String> {
    let text = input.get(key)?.as_str()?;
    let single_line = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if PATH_KEYS.contains(&key) {
        return Some(file_name_of(&single_line));
    }
    Some(single_line)
}

fn file_name_of(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map_or_else(|| path.to_owned(), |name| name.to_string_lossy().into_owned())
}

fn truncate(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_owned();
    }
    let kept: String = text.chars().take(limit).collect();
    format!("{kept}…")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn summarizes_bash_command_on_one_line() {
        let input = json!({ "command": "npm  run\nbuild" });
        assert_eq!(summarize_tool_input(&input), "npm run build");
    }

    #[test]
    fn summarizes_file_tools_by_file_name() {
        let input = json!({ "file_path": r"C:\code\app\main.rs" });
        assert_eq!(summarize_tool_input(&input), "main.rs");
    }

    #[test]
    fn truncates_long_summaries() {
        let input = json!({ "command": "x".repeat(200) });
        assert_eq!(summarize_tool_input(&input).chars().count(), SUMMARY_LIMIT + 1);
    }

    #[test]
    fn previews_non_command_input_as_json() {
        let input = json!({ "pattern": "todo" });
        assert!(preview_tool_input(&input).contains("\"pattern\""));
    }

    #[test]
    fn invalid_body_yields_defaults() {
        let payload = HookPayload::parse(b"not json");
        assert_eq!(payload.session_id(), "default");
        assert_eq!(payload.tool_name(), "tool");
    }
}
