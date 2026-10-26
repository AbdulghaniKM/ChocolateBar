use std::collections::VecDeque;
use std::path::Path;

use serde::Serialize;

use crate::core::activity::now_ms;

const ENTRY_LIMIT: usize = 40;
const IMAGE_LIMIT: usize = 8;
const PREVIEW_CHARS: usize = 600;

pub enum Content {
    Text(String),
    Image { dib: Vec<u8>, thumbnail: String, width: u32, height: u32 },
    Files(Vec<String>),
}

impl Content {
    fn signature(&self) -> String {
        match self {
            Self::Text(text) => format!("t:{text}"),
            Self::Files(paths) => format!("f:{}", paths.join("\n")),
            Self::Image { dib, .. } => format!("i:{}:{:016x}", dib.len(), fnv1a(dib)),
        }
    }

    fn is_image(&self) -> bool {
        matches!(self, Self::Image { .. })
    }
}

pub struct Entry {
    pub id: u64,
    pub content: Content,
    pub pinned: bool,
    captured_at_ms: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileView {
    name: String,
    path: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryView {
    id: u64,
    kind: &'static str,
    text: Option<String>,
    thumbnail: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    files: Option<Vec<FileView>>,
    pinned: bool,
    captured_at_ms: u64,
}

#[derive(Default)]
pub struct History {
    entries: VecDeque<Entry>,
    next_id: u64,
}

impl History {
    /// Adds new content at the top; copying something already in the history just moves it up.
    pub fn record(&mut self, content: Content) {
        let signature = content.signature();
        if let Some(position) = self.entries.iter().position(|entry| entry.content.signature() == signature) {
            if let Some(mut existing) = self.entries.remove(position) {
                existing.captured_at_ms = now_ms();
                self.entries.push_front(existing);
            }
            return;
        }
        self.next_id += 1;
        self.entries.push_front(Entry { id: self.next_id, content, pinned: false, captured_at_ms: now_ms() });
        self.trim();
    }

    pub fn get(&self, id: u64) -> Option<&Entry> {
        self.entries.iter().find(|entry| entry.id == id)
    }

    pub fn promote(&mut self, id: u64) {
        if let Some(position) = self.entries.iter().position(|entry| entry.id == id) {
            if let Some(mut entry) = self.entries.remove(position) {
                entry.captured_at_ms = now_ms();
                self.entries.push_front(entry);
            }
        }
    }

    pub fn set_pinned(&mut self, id: u64, pinned: bool) {
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.id == id) {
            entry.pinned = pinned;
        }
    }

    pub fn remove(&mut self, id: u64) {
        self.entries.retain(|entry| entry.id != id);
    }

    pub fn clear_unpinned(&mut self) {
        self.entries.retain(|entry| entry.pinned);
    }

    pub fn view(&self) -> Vec<EntryView> {
        let (pinned, recent): (Vec<_>, Vec<_>) = self.entries.iter().partition(|entry| entry.pinned);
        pinned.into_iter().chain(recent).map(view_of).collect()
    }

    fn trim(&mut self) {
        let mut images = 0;
        let mut kept = 0;
        self.entries.retain(|entry| {
            if entry.pinned {
                return true;
            }
            let image_over_limit = entry.content.is_image() && {
                images += 1;
                images > IMAGE_LIMIT
            };
            kept += 1;
            kept <= ENTRY_LIMIT && !image_over_limit
        });
    }
}

fn view_of(entry: &Entry) -> EntryView {
    let mut view = EntryView {
        id: entry.id,
        kind: "text",
        text: None,
        thumbnail: None,
        width: None,
        height: None,
        files: None,
        pinned: entry.pinned,
        captured_at_ms: entry.captured_at_ms,
    };
    match &entry.content {
        Content::Text(text) => view.text = Some(text.chars().take(PREVIEW_CHARS).collect()),
        Content::Image { thumbnail, width, height, .. } => {
            view.kind = "image";
            view.thumbnail = Some(thumbnail.clone());
            view.width = Some(*width);
            view.height = Some(*height);
        }
        Content::Files(paths) => {
            view.kind = "files";
            view.files = Some(paths.iter().map(|path| FileView { name: file_name(path), path: path.clone() }).collect());
        }
    }
    view
}

fn file_name(path: &str) -> String {
    Path::new(path).file_name().map_or_else(|| path.to_string(), |name| name.to_string_lossy().into_owned())
}

fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| (hash ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(seed: u8) -> Content {
        Content::Image { dib: vec![seed; 16], thumbnail: String::new(), width: 1, height: 1 }
    }

    #[test]
    fn copying_the_same_text_again_moves_it_to_the_top() {
        let mut history = History::default();
        history.record(Content::Text("a".into()));
        history.record(Content::Text("b".into()));
        history.record(Content::Text("a".into()));
        let ids: Vec<u64> = history.entries.iter().map(|entry| entry.id).collect();
        assert_eq!(ids, vec![1, 2]);
    }

    #[test]
    fn pinned_entries_survive_clearing_and_are_listed_first() {
        let mut history = History::default();
        history.record(Content::Text("keep".into()));
        history.record(Content::Text("drop".into()));
        history.set_pinned(1, true);
        assert_eq!(history.view()[0].id, 1);
        history.clear_unpinned();
        assert_eq!(history.entries.len(), 1);
    }

    #[test]
    fn caps_unpinned_images_and_total_entries() {
        let mut history = History::default();
        for seed in 0..12 {
            history.record(image(seed));
        }
        assert_eq!(history.entries.len(), IMAGE_LIMIT);
        for index in 0..60 {
            history.record(Content::Text(index.to_string()));
        }
        assert_eq!(history.entries.len(), ENTRY_LIMIT);
    }

    #[test]
    fn files_are_listed_by_name() {
        let mut history = History::default();
        history.record(Content::Files(vec!["C:\\Users\\me\\report.docx".into()]));
        let view = serde_json::to_value(history.view()).unwrap();
        assert_eq!(view[0]["files"][0]["name"], "report.docx");
    }
}
