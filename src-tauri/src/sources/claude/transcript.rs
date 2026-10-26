use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use serde_json::Value;

const INTERRUPT_MARKER: &str = "[Request interrupted by user";
const MAX_TAIL_BYTES: u64 = 512 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranscriptChange {
    Unchanged,
    Grew,
    Interrupted,
}

pub struct TranscriptWatcher {
    path: PathBuf,
    offset: u64,
    model: Option<String>,
}

impl TranscriptWatcher {
    pub fn starting_at_end(path: PathBuf) -> Self {
        let offset = file_length(&path).unwrap_or(0);
        let model = recent_model(&path, offset);
        Self { path, offset, model }
    }

    pub fn model(&self) -> Option<&str> {
        self.model.as_deref()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn skip_to_end(&mut self) {
        self.offset = file_length(&self.path).unwrap_or(0);
    }

    pub fn poll(&mut self) -> TranscriptChange {
        let Some(length) = file_length(&self.path) else {
            return TranscriptChange::Unchanged;
        };
        if length < self.offset {
            self.offset = length;
            return TranscriptChange::Grew;
        }
        if length == self.offset {
            return TranscriptChange::Unchanged;
        }
        let tail = self.take_complete_lines(length);
        if let Some(model) = newest_assistant_model(&tail) {
            self.model = Some(model);
        }
        if newest_user_entry_is_interrupt(&tail) {
            TranscriptChange::Interrupted
        } else {
            TranscriptChange::Grew
        }
    }

    fn take_complete_lines(&mut self, length: u64) -> String {
        let start = self.offset.max(length.saturating_sub(MAX_TAIL_BYTES));
        let bytes = read_range(&self.path, start, length).unwrap_or_default();
        let complete_len = bytes.iter().rposition(|byte| *byte == b'\n').map_or(0, |index| index + 1);
        self.offset = if complete_len == 0 && length - start >= MAX_TAIL_BYTES {
            length
        } else {
            start + complete_len as u64
        };
        String::from_utf8_lossy(&bytes[..complete_len]).into_owned()
    }
}

pub fn newest_user_entry_is_interrupt(lines: &str) -> bool {
    lines
        .lines()
        .rev()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|entry| entry.get("type").and_then(Value::as_str) == Some("user"))
        .is_some_and(|entry| user_message_text(&entry).contains(INTERRUPT_MARKER))
}

pub fn newest_assistant_model(lines: &str) -> Option<String> {
    lines
        .lines()
        .rev()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|entry| entry.get("type").and_then(Value::as_str) == Some("assistant"))
        .find_map(|entry| real_model_name(&entry))
}

fn real_model_name(entry: &Value) -> Option<String> {
    let name = entry.pointer("/message/model")?.as_str()?;
    (!name.is_empty() && !name.starts_with('<')).then(|| name.to_owned())
}

fn recent_model(path: &Path, length: u64) -> Option<String> {
    let start = length.saturating_sub(MAX_TAIL_BYTES);
    let bytes = read_range(path, start, length)?;
    let text = String::from_utf8_lossy(&bytes);
    let complete = if start > 0 { text.split_once('\n').map_or("", |(_, rest)| rest) } else { &text };
    newest_assistant_model(complete)
}

fn user_message_text(entry: &Value) -> String {
    match entry.pointer("/message/content") {
        Some(Value::String(text)) => text.clone(),
        Some(Value::Array(blocks)) => blocks
            .iter()
            .filter_map(|block| block.get("text").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

fn file_length(path: &Path) -> Option<u64> {
    fs::metadata(path).ok().map(|metadata| metadata.len())
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
    use std::io::Write;

    use super::*;

    const INTERRUPT_LINE: &str = r#"{"parentUuid":"a1","isSidechain":false,"promptId":"p1","type":"user","message":{"role":"user","content":[{"type":"text","text":"[Request interrupted by user]"}]},"uuid":"u2","timestamp":"2026-10-26T10:00:00.000Z","userType":"external","cwd":"C:\\code","sessionId":"s1"}"#;
    const TOOL_INTERRUPT_LINE: &str = r#"{"type":"user","message":{"role":"user","content":[{"type":"text","text":"[Request interrupted by user for tool use]"}]}}"#;
    const PROMPT_LINE: &str = r#"{"type":"user","message":{"role":"user","content":"fix the bug"}}"#;
    const ASSISTANT_LINE: &str = r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"working"}]}}"#;
    const BLOB_LINE: &str = r#"{"type":"api-request-blob","message":{"role":"user","content":[{"type":"text","text":"[Request interrupted by user]"}]}}"#;

    fn temp_transcript(name: &str) -> PathBuf {
        let folder = std::env::temp_dir().join(format!("chocolatebar-transcript-{name}-{}", std::process::id()));
        fs::create_dir_all(&folder).unwrap();
        folder.join("session.jsonl")
    }

    fn append(path: &Path, text: &str) {
        let mut file = fs::OpenOptions::new().create(true).append(true).open(path).unwrap();
        file.write_all(text.as_bytes()).unwrap();
    }

    fn cleanup(path: &Path) {
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn real_interrupt_line_is_detected() {
        assert!(newest_user_entry_is_interrupt(INTERRUPT_LINE));
        assert!(newest_user_entry_is_interrupt(TOOL_INTERRUPT_LINE));
    }

    #[test]
    fn interrupt_followed_by_a_new_prompt_is_not_an_interrupt() {
        let lines = format!("{INTERRUPT_LINE}\n{PROMPT_LINE}\n");
        assert!(!newest_user_entry_is_interrupt(&lines));
    }

    #[test]
    fn assistant_output_after_the_interrupt_does_not_hide_it() {
        let lines = format!("{PROMPT_LINE}\n{INTERRUPT_LINE}\n{ASSISTANT_LINE}\n");
        assert!(newest_user_entry_is_interrupt(&lines));
    }

    #[test]
    fn non_user_entries_and_garbage_are_ignored() {
        assert!(!newest_user_entry_is_interrupt(&format!("{BLOB_LINE}\nnot json\n{ASSISTANT_LINE}")));
        assert!(!newest_user_entry_is_interrupt(""));
    }

    #[test]
    fn watcher_reports_only_what_was_appended_after_it_started() {
        let path = temp_transcript("append");
        append(&path, &format!("{INTERRUPT_LINE}\n"));
        let mut watcher = TranscriptWatcher::starting_at_end(path.clone());
        assert_eq!(watcher.poll(), TranscriptChange::Unchanged);
        append(&path, &format!("{ASSISTANT_LINE}\n"));
        assert_eq!(watcher.poll(), TranscriptChange::Grew);
        append(&path, &format!("{INTERRUPT_LINE}\n"));
        assert_eq!(watcher.poll(), TranscriptChange::Interrupted);
        assert_eq!(watcher.poll(), TranscriptChange::Unchanged);
        cleanup(&path);
    }

    #[test]
    fn watcher_waits_for_the_line_to_be_complete() {
        let path = temp_transcript("partial");
        append(&path, "");
        let mut watcher = TranscriptWatcher::starting_at_end(path.clone());
        let (head, rest) = INTERRUPT_LINE.split_at(40);
        append(&path, head);
        assert_eq!(watcher.poll(), TranscriptChange::Grew);
        append(&path, &format!("{rest}\n"));
        assert_eq!(watcher.poll(), TranscriptChange::Interrupted);
        cleanup(&path);
    }

    #[test]
    fn skipping_to_the_end_forgets_an_older_interrupt() {
        let path = temp_transcript("skip");
        append(&path, "");
        let mut watcher = TranscriptWatcher::starting_at_end(path.clone());
        append(&path, &format!("{INTERRUPT_LINE}\n"));
        watcher.skip_to_end();
        assert_eq!(watcher.poll(), TranscriptChange::Unchanged);
        cleanup(&path);
    }

    #[test]
    fn newest_assistant_model_is_picked_and_synthetic_ones_skipped() {
        let older = r#"{"type":"assistant","message":{"model":"claude-sonnet-5","role":"assistant","content":[]}}"#;
        let newer = r#"{"type":"assistant","message":{"model":"claude-opus-5-5","role":"assistant","content":[]}}"#;
        let synthetic = r#"{"type":"assistant","message":{"model":"<synthetic>","role":"assistant","content":[]}}"#;
        let lines = format!("{older}\n{newer}\n{synthetic}\n{PROMPT_LINE}\n");
        assert_eq!(newest_assistant_model(&lines).as_deref(), Some("claude-opus-5-5"));
        assert_eq!(newest_assistant_model(PROMPT_LINE), None);
    }

    #[test]
    fn watcher_knows_the_model_from_history_and_follows_changes() {
        let path = temp_transcript("model");
        let assistant = |model: &str| format!(r#"{{"type":"assistant","message":{{"model":"{model}","content":[]}}}}"#);
        append(&path, &format!("{}\n", assistant("claude-sonnet-5")));
        let mut watcher = TranscriptWatcher::starting_at_end(path.clone());
        assert_eq!(watcher.model(), Some("claude-sonnet-5"));
        append(&path, &format!("{}\n", assistant("claude-opus-5-5")));
        watcher.poll();
        assert_eq!(watcher.model(), Some("claude-opus-5-5"));
        cleanup(&path);
    }

    #[test]
    fn missing_file_is_quiet() {
        let path = temp_transcript("missing");
        let mut watcher = TranscriptWatcher::starting_at_end(path.clone());
        assert_eq!(watcher.poll(), TranscriptChange::Unchanged);
        cleanup(&path);
    }
}
