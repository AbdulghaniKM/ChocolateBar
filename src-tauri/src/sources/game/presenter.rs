use serde::Serialize;

use crate::core::activity::{Activity, ActivityKind, CompactView, Priority};

use super::model::GameDownload;
use super::playtime::PlayingSession;

const SOURCE_NAME: &str = "game";
const PLAYING_KEY: &str = "playing";
const DOWNLOAD_ACCENT: &str = "#38bdf8";
const PLAYING_ACCENT: &str = "#22c55e";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadView<'a> {
    name: &'a str,
    launcher: &'static str,
    fraction: Option<f32>,
    paused: bool,
    downloaded_bytes: Option<u64>,
    total_bytes: Option<u64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PlayingView<'a> {
    name: &'a str,
    started_at_ms: u64,
    session_seconds: u64,
    total_seconds: u64,
}

pub fn download_key(id: &str) -> String {
    format!("download:{id}")
}

pub fn playing_key() -> &'static str {
    PLAYING_KEY
}

pub fn download_activity(download: &GameDownload) -> Activity {
    let view = DownloadView {
        name: &download.name,
        launcher: download.launcher.label(),
        fraction: download.fraction(),
        paused: download.paused,
        downloaded_bytes: download.downloaded_bytes,
        total_bytes: download.total_bytes,
    };
    let mut compact = CompactView::new("\u{1F3AE}", download.name.clone())
        .subtitle(download_status(download))
        .accent(DOWNLOAD_ACCENT);
    if let Some(fraction) = download.fraction() {
        compact = compact.progress(fraction);
    }
    let expanded = serde_json::to_value(view).unwrap_or_default();
    Activity::new(SOURCE_NAME, &download_key(&download.id), ActivityKind::Game, compact)
        .expanded(expanded)
        .priority(Priority::Normal)
}

pub fn playing_activity(session: &PlayingSession, total_seconds: u64) -> Activity {
    let compact = CompactView::new("\u{1F579}\u{FE0F}", session.name.clone())
        .subtitle(format!("Playing \u{00B7} {}", duration_text(session.elapsed.as_secs())))
        .accent(PLAYING_ACCENT);
    let view = PlayingView {
        name: &session.name,
        started_at_ms: session.started_at_ms,
        session_seconds: session.elapsed.as_secs(),
        total_seconds,
    };
    let expanded = serde_json::to_value(view).unwrap_or_default();
    Activity::new(SOURCE_NAME, PLAYING_KEY, ActivityKind::Game, compact)
        .expanded(expanded)
        .priority(Priority::Ambient)
}

fn download_status(download: &GameDownload) -> String {
    let launcher = download.launcher.label();
    match (download.fraction(), download.paused) {
        (Some(fraction), true) => format!("{launcher} \u{00B7} paused {}%", percent(fraction)),
        (Some(fraction), false) => format!("{launcher} \u{00B7} {}%", percent(fraction)),
        (None, _) => format!("{launcher} \u{00B7} installing"),
    }
}

fn percent(fraction: f32) -> u32 {
    (fraction * 100.0).round() as u32
}

fn duration_text(seconds: u64) -> String {
    let (hours, minutes) = (seconds / 3600, seconds % 3600 / 60);
    if hours > 0 {
        format!("{hours}h {minutes:02}m")
    } else {
        format!("{minutes}m")
    }
}
