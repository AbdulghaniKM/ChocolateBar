use serde_json::json;

use super::files::ShelfFiles;
use crate::core::activity::{ActionSpec, ActionStyle, Activity, ActivityKind, CompactView, Priority};
use crate::core::source::SourceContext;

pub const FILES_KEY: &str = "files";
pub const DROP_KEY: &str = "drop";
const DROP_HOVER_TTL_MS: u64 = 15_000;
const SHELF_ACCENT: &str = "#a78bfa";

pub fn files_activity(ctx: &SourceContext, files: &ShelfFiles, just_dropped: bool) -> Activity {
    let entries = files.entries();
    let first_name = entries.first().map(|entry| entry.name.clone()).unwrap_or_default();
    let compact = CompactView::new("📎", count_label(files.len()))
        .subtitle(first_name)
        .accent(SHELF_ACCENT)
        .badge(files.len().to_string());
    let priority = if just_dropped { Priority::High } else { Priority::Normal };
    ctx.activity(FILES_KEY, ActivityKind::Shelf, compact)
        .priority(priority)
        .expanded(json!({ "view": "files", "files": entries }))
        .action(ActionSpec::button("clear", "Clear shelf").icon("🗑").style(ActionStyle::Danger))
}

pub fn drop_hover_activity(ctx: &SourceContext) -> Activity {
    let compact = CompactView::new("📥", "Drop to shelve").subtitle("Release to keep files here").accent(SHELF_ACCENT);
    ctx.activity(DROP_KEY, ActivityKind::Shelf, compact)
        .priority(Priority::Critical)
        .alert(true)
        .ttl_ms(DROP_HOVER_TTL_MS)
        .expanded(json!({ "view": "drop" }))
}

fn count_label(count: usize) -> String {
    match count {
        1 => "1 file".to_owned(),
        _ => format!("{count} files"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_label_pluralizes() {
        assert_eq!(count_label(1), "1 file");
        assert_eq!(count_label(4), "4 files");
    }
}
