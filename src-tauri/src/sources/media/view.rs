use serde::Serialize;
use serde_json::Value;

use super::lrc::LyricLine;
use super::session::{Playback, RepeatMode, SourceSummary};
use crate::core::activity::{now_ms, Activity, ActivityKind, CompactView, Priority};
use crate::core::source::SourceContext;

const MEDIA_ICON: &str = "🎵";
pub const ACTIVITY_KEY: &str = "current";

pub struct TrackFrame<'a> {
    pub title: &'a str,
    pub artist: &'a str,
    pub album: &'a str,
    pub art: Option<&'a str>,
    pub lyrics: &'a [LyricLine],
    pub playback: &'a Playback,
    pub source: &'a SourceSummary,
    pub sources: &'a [SourceSummary],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExpandedPayload<'a> {
    title: &'a str,
    artist: &'a str,
    album: &'a str,
    art: Option<&'a str>,
    playing: bool,
    position_ms: u64,
    duration_ms: u64,
    captured_at_ms: u64,
    rate: f64,
    can_go_previous: bool,
    can_go_next: bool,
    lyrics: &'a [LyricLine],
    shuffle: Option<bool>,
    repeat: Option<RepeatMode>,
    source: &'a SourceSummary,
    sources: &'a [SourceSummary],
}

pub fn build_activity(ctx: &SourceContext, frame: &TrackFrame) -> Activity {
    ctx.activity(ACTIVITY_KEY, ActivityKind::Media, compact_view(frame))
        .priority(priority_for(frame.playback.playing))
        .expanded(serde_json::to_value(expanded_payload(frame)).unwrap_or(Value::Null))
}

fn priority_for(playing: bool) -> Priority {
    if playing {
        Priority::Normal
    } else {
        Priority::Ambient
    }
}

fn compact_view(frame: &TrackFrame) -> CompactView {
    let view = CompactView::new(MEDIA_ICON, frame.title);
    if frame.artist.is_empty() {
        view.subtitle(frame.source.app.as_str())
    } else {
        view.subtitle(frame.artist)
    }
}

fn expanded_payload<'a>(frame: &TrackFrame<'a>) -> ExpandedPayload<'a> {
    let playback = frame.playback;
    ExpandedPayload {
        title: frame.title,
        artist: frame.artist,
        album: frame.album,
        art: frame.art,
        playing: playback.playing,
        position_ms: playback.position_ms,
        duration_ms: playback.duration_ms,
        captured_at_ms: now_ms(),
        rate: playback.rate,
        can_go_previous: playback.can_go_previous,
        can_go_next: playback.can_go_next,
        lyrics: frame.lyrics,
        shuffle: playback.shuffle,
        repeat: playback.repeat,
        source: frame.source,
        sources: frame.sources,
    }
}
