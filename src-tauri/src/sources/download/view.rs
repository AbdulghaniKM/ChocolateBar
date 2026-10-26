use serde::Serialize;
use serde_json::json;

use crate::core::activity::{Activity, ActivityKind, CompactView, Priority};
use crate::core::source::SourceContext;

use super::tracker::DownloadSnapshot;
use super::units::{format_bytes, format_rate};

const COMPLETED_TTL_MS: u64 = 4_000;
const ACCENT_ACTIVE: &str = "#38bdf8";
const ACCENT_DONE: &str = "#22c55e";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadView<'a> {
    file_name: &'a str,
    bytes_done: u64,
    size_text: String,
    bytes_per_second: f64,
    speed_text: String,
    elapsed_seconds: u64,
    done: bool,
    indeterminate: bool,
}

pub fn progress_activity(ctx: &SourceContext, snapshot: &DownloadSnapshot) -> Activity {
    let subtitle = format!("{} · {}", format_bytes(snapshot.bytes_done), format_rate(snapshot.bytes_per_second));
    let compact = CompactView::new("⬇", snapshot.file_name.clone()).subtitle(subtitle).accent(ACCENT_ACTIVE);
    base_activity(ctx, snapshot, compact, false).priority(Priority::Normal)
}

pub fn completed_activity(ctx: &SourceContext, snapshot: &DownloadSnapshot) -> Activity {
    let compact = CompactView::new("✅", snapshot.file_name.clone())
        .subtitle(format!("Download complete · {}", format_bytes(snapshot.bytes_done)))
        .progress(1.0)
        .accent(ACCENT_DONE);
    base_activity(ctx, snapshot, compact, true)
        .priority(Priority::High)
        .alert(true)
        .ttl_ms(COMPLETED_TTL_MS)
}

fn base_activity(ctx: &SourceContext, snapshot: &DownloadSnapshot, compact: CompactView, done: bool) -> Activity {
    let view = DownloadView {
        file_name: &snapshot.file_name,
        bytes_done: snapshot.bytes_done,
        size_text: format_bytes(snapshot.bytes_done),
        bytes_per_second: snapshot.bytes_per_second,
        speed_text: format_rate(snapshot.bytes_per_second),
        elapsed_seconds: snapshot.elapsed.as_secs(),
        done,
        indeterminate: !done,
    };
    ctx.activity(&snapshot.key, ActivityKind::Download, compact).expanded(json!(view))
}
