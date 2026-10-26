use serde::Serialize;
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession as Session,
    GlobalSystemMediaTransportControlsSessionManager as Manager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as PlaybackStatus,
};
use windows::Media::MediaPlaybackAutoRepeatMode;
use windows::Storage::Streams::IRandomAccessStreamReference;
use windows_core::Result;

use crate::core::activity::now_ms;

const TICKS_PER_MILLISECOND: i64 = 10_000;
const WINDOWS_TO_UNIX_EPOCH_MS: i64 = 11_644_473_600_000;
const MAX_EXTRAPOLATION_MS: i64 = 6 * 60 * 60 * 1000;
const FIREFOX_AUMID: &str = "308046b0af4a39cb";

pub struct Track {
    pub title: String,
    pub artist: String,
    pub album: String,
}

impl Track {
    pub fn key(&self) -> String {
        format!("{}\u{1f}{}\u{1f}{}", self.artist, self.title, self.album)
    }

    pub fn is_blank(&self) -> bool {
        self.title.trim().is_empty() && self.artist.trim().is_empty()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RepeatMode {
    Off,
    Track,
    List,
}

pub struct Playback {
    pub playing: bool,
    pub can_go_previous: bool,
    pub can_go_next: bool,
    pub position_ms: u64,
    pub duration_ms: u64,
    pub rate: f64,
    pub shuffle: Option<bool>,
    pub repeat: Option<RepeatMode>,
}

#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSummary {
    pub id: String,
    pub app: String,
    pub kind: &'static str,
    pub playing: bool,
}

pub struct Reading {
    pub source: SourceSummary,
    pub sources: Vec<SourceSummary>,
    pub track: Track,
    pub playback: Playback,
    pub thumbnail: Option<IRandomAccessStreamReference>,
}

pub enum Control {
    Previous,
    TogglePlayPause,
    Next,
    SeekTo(u64),
    SetShuffle(bool),
    SetRepeat(RepeatMode),
}

pub fn open_manager() -> Result<Manager> {
    Manager::RequestAsync()?.join()
}

pub fn sessions(manager: &Manager) -> Vec<(SourceSummary, Session)> {
    let Ok(list) = manager.GetSessions() else { return Vec::new() };
    list.into_iter().filter_map(|session| Some((summarize(&session)?, session))).collect()
}

pub fn current_session_id(manager: &Manager) -> Option<String> {
    manager.GetCurrentSession().ok()?.SourceAppUserModelId().ok().map(|id| id.to_string())
}

pub fn read(session: &Session, source: SourceSummary, sources: Vec<SourceSummary>) -> Result<Reading> {
    let properties = session.TryGetMediaPropertiesAsync()?.join()?;
    let track = Track {
        title: properties.Title()?.to_string(),
        artist: properties.Artist()?.to_string(),
        album: properties.AlbumTitle()?.to_string(),
    };
    Ok(Reading {
        source,
        sources,
        track,
        playback: read_playback(session)?,
        thumbnail: properties.Thumbnail().ok(),
    })
}

pub fn send(session: &Session, control: Control) -> Result<bool> {
    match control {
        Control::Previous => session.TrySkipPreviousAsync()?.join(),
        Control::TogglePlayPause => session.TryTogglePlayPauseAsync()?.join(),
        Control::Next => session.TrySkipNextAsync()?.join(),
        Control::SeekTo(position_ms) => {
            let ticks = (position_ms as i64).saturating_mul(TICKS_PER_MILLISECOND);
            session.TryChangePlaybackPositionAsync(ticks)?.join()
        }
        Control::SetShuffle(active) => session.TryChangeShuffleActiveAsync(active)?.join(),
        Control::SetRepeat(mode) => session.TryChangeAutoRepeatModeAsync(to_windows_repeat(mode))?.join(),
    }
}

fn summarize(session: &Session) -> Option<SourceSummary> {
    let id = session.SourceAppUserModelId().ok()?.to_string();
    let playing = session
        .GetPlaybackInfo()
        .and_then(|info| info.PlaybackStatus())
        .is_ok_and(|status| status == PlaybackStatus::Playing);
    let (app, kind) = describe_app(&id);
    Some(SourceSummary { id, app, kind, playing })
}

fn describe_app(aumid: &str) -> (String, &'static str) {
    let lower = aumid.to_lowercase();
    const KNOWN: [(&str, &str, &str); 8] = [
        ("spotify", "Spotify", "spotify"),
        ("brave", "Brave", "browser"),
        ("chrome", "Chrome", "browser"),
        ("msedge", "Edge", "browser"),
        ("firefox", "Firefox", "browser"),
        (FIREFOX_AUMID, "Firefox", "browser"),
        ("opera", "Opera", "browser"),
        ("vivaldi", "Vivaldi", "browser"),
    ];
    KNOWN
        .iter()
        .find(|(needle, _, _)| lower.contains(needle))
        .map(|(_, label, kind)| ((*label).to_string(), *kind))
        .unwrap_or_else(|| (readable_app_name(aumid), "app"))
}

fn readable_app_name(aumid: &str) -> String {
    let segment = aumid.rsplit('!').next().unwrap_or(aumid);
    let segment = segment.rsplit('\\').next().unwrap_or(segment);
    let name = segment.trim_end_matches(".exe").trim_end_matches(".EXE");
    let mut characters = name.chars();
    match characters.next() {
        Some(first) => first.to_uppercase().chain(characters).collect(),
        None => "Media".into(),
    }
}

fn read_playback(session: &Session) -> Result<Playback> {
    let info = session.GetPlaybackInfo()?;
    let timeline = session.GetTimelineProperties()?;
    let controls = info.Controls()?;
    let playing = info.PlaybackStatus()? == PlaybackStatus::Playing;
    let rate = info.PlaybackRate().and_then(|rate| rate.Value()).unwrap_or(1.0);
    let duration_ticks = timeline.EndTime()?.Duration - timeline.StartTime()?.Duration;
    let duration_ms = ticks_to_ms(duration_ticks);
    let reported_ms = ticks_to_ms(timeline.Position()?.Duration);
    let stale_ms = if playing { elapsed_since(timeline.LastUpdatedTime()?.UniversalTime) } else { 0 };
    let position_ms = reported_ms + (stale_ms as f64 * rate) as u64;
    let shuffle_supported = controls.IsShuffleEnabled().unwrap_or(false);
    let repeat_supported = controls.IsRepeatEnabled().unwrap_or(false);
    Ok(Playback {
        playing,
        can_go_previous: controls.IsPreviousEnabled().unwrap_or(false),
        can_go_next: controls.IsNextEnabled().unwrap_or(false),
        position_ms: clamp_to_duration(position_ms, duration_ms),
        duration_ms,
        rate,
        shuffle: shuffle_supported
            .then(|| info.IsShuffleActive().and_then(|value| value.Value()).unwrap_or(false)),
        repeat: repeat_supported.then(|| {
            info.AutoRepeatMode()
                .and_then(|value| value.Value())
                .map(from_windows_repeat)
                .unwrap_or(RepeatMode::Off)
        }),
    })
}

fn from_windows_repeat(mode: MediaPlaybackAutoRepeatMode) -> RepeatMode {
    match mode {
        MediaPlaybackAutoRepeatMode::Track => RepeatMode::Track,
        MediaPlaybackAutoRepeatMode::List => RepeatMode::List,
        _ => RepeatMode::Off,
    }
}

fn to_windows_repeat(mode: RepeatMode) -> MediaPlaybackAutoRepeatMode {
    match mode {
        RepeatMode::Off => MediaPlaybackAutoRepeatMode::None,
        RepeatMode::Track => MediaPlaybackAutoRepeatMode::Track,
        RepeatMode::List => MediaPlaybackAutoRepeatMode::List,
    }
}

pub fn parse_repeat(value: &str) -> Option<RepeatMode> {
    match value {
        "off" => Some(RepeatMode::Off),
        "track" => Some(RepeatMode::Track),
        "list" => Some(RepeatMode::List),
        _ => None,
    }
}

fn ticks_to_ms(ticks: i64) -> u64 {
    (ticks / TICKS_PER_MILLISECOND).max(0) as u64
}

fn elapsed_since(universal_ticks: i64) -> u64 {
    let updated_unix_ms = universal_ticks / TICKS_PER_MILLISECOND - WINDOWS_TO_UNIX_EPOCH_MS;
    (now_ms() as i64 - updated_unix_ms).clamp(0, MAX_EXTRAPOLATION_MS) as u64
}

fn clamp_to_duration(position_ms: u64, duration_ms: u64) -> u64 {
    if duration_ms == 0 {
        position_ms
    } else {
        position_ms.min(duration_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_known_players_and_browsers() {
        assert_eq!(describe_app("Spotify.exe"), ("Spotify".into(), "spotify"));
        assert_eq!(describe_app("Chrome"), ("Chrome".into(), "browser"));
        assert_eq!(describe_app("Brave"), ("Brave".into(), "browser"));
        assert_eq!(describe_app("308046B0AF4A39CB"), ("Firefox".into(), "browser"));
    }

    #[test]
    fn falls_back_to_the_executable_name() {
        assert_eq!(describe_app("vlc.exe"), ("Vlc".into(), "app"));
        assert_eq!(describe_app("Microsoft.ZuneMusic_8wekyb3d8bbwe!Microsoft.ZuneMusic").1, "app");
    }
}
