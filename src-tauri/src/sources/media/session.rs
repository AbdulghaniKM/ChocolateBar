use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession as Session,
    GlobalSystemMediaTransportControlsSessionManager as Manager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as PlaybackStatus,
};
use windows::Storage::Streams::IRandomAccessStreamReference;
use windows_core::Result;

use crate::core::activity::now_ms;

const TICKS_PER_MILLISECOND: i64 = 10_000;
const WINDOWS_TO_UNIX_EPOCH_MS: i64 = 11_644_473_600_000;
const MAX_EXTRAPOLATION_MS: i64 = 6 * 60 * 60 * 1000;

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

pub struct Playback {
    pub playing: bool,
    pub can_go_previous: bool,
    pub can_go_next: bool,
    pub position_ms: u64,
    pub duration_ms: u64,
    pub rate: f64,
}

pub struct Reading {
    pub track: Track,
    pub playback: Playback,
    pub thumbnail: Option<IRandomAccessStreamReference>,
}

pub enum Control {
    Previous,
    TogglePlayPause,
    Next,
    SeekTo(u64),
}

pub fn open_manager() -> Result<Manager> {
    Manager::RequestAsync()?.join()
}

pub fn current_session(manager: &Manager) -> Option<Session> {
    manager.GetCurrentSession().ok()
}

pub fn read(session: &Session) -> Result<Reading> {
    let properties = session.TryGetMediaPropertiesAsync()?.join()?;
    let track = Track {
        title: properties.Title()?.to_string(),
        artist: properties.Artist()?.to_string(),
        album: properties.AlbumTitle()?.to_string(),
    };
    Ok(Reading {
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
    Ok(Playback {
        playing,
        can_go_previous: controls.IsPreviousEnabled().unwrap_or(false),
        can_go_next: controls.IsNextEnabled().unwrap_or(false),
        position_ms: clamp_to_duration(position_ms, duration_ms),
        duration_ms,
        rate,
    })
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
