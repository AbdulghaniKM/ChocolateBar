use std::time::{Duration, Instant};

use super::artwork;
use super::lrc::LyricLine;
use super::lyrics::{LyricsQuery, LyricsService};
use super::playback_flag;
use super::session::{Playback, Reading, RepeatMode, SourceSummary, Track};
use super::view::{build_activity, TrackFrame, ACTIVITY_KEY};
use crate::core::settings::Settings;
use crate::core::source::SourceContext;

const SEEK_TOLERANCE_MS: f64 = 1_500.0;
const PAUSED_LINGER: Duration = Duration::from_secs(30);
const MAX_ARTWORK_ATTEMPTS: u8 = 6;

struct TrackState {
    key: String,
    art: Option<String>,
    art_attempts: u8,
    lyrics: Vec<LyricLine>,
}

struct Published {
    playing: bool,
    can_go_previous: bool,
    can_go_next: bool,
    duration_ms: u64,
    position_ms: u64,
    rate: f64,
    shuffle: Option<bool>,
    repeat: Option<RepeatMode>,
    sources: Vec<SourceSummary>,
    at: Instant,
}

impl Published {
    fn of(reading: &Reading) -> Self {
        let playback = &reading.playback;
        Self {
            playing: playback.playing,
            can_go_previous: playback.can_go_previous,
            can_go_next: playback.can_go_next,
            duration_ms: playback.duration_ms,
            position_ms: playback.position_ms,
            rate: playback.rate,
            shuffle: playback.shuffle,
            repeat: playback.repeat,
            sources: reading.sources.clone(),
            at: Instant::now(),
        }
    }

    fn expected_position_ms(&self) -> f64 {
        let elapsed_ms = self.at.elapsed().as_millis() as f64;
        let advanced = if self.playing { elapsed_ms * self.rate } else { 0.0 };
        self.position_ms as f64 + advanced
    }

    fn differs_from(&self, reading: &Reading) -> bool {
        let playback = &reading.playback;
        self.playing != playback.playing
            || self.shuffle != playback.shuffle
            || self.repeat != playback.repeat
            || self.sources != reading.sources
            || self.can_go_previous != playback.can_go_previous
            || self.can_go_next != playback.can_go_next
            || self.duration_ms != playback.duration_ms
            || (self.expected_position_ms() - playback.position_ms as f64).abs() > SEEK_TOLERANCE_MS
    }
}

#[derive(Default)]
pub struct Publisher {
    lyrics: LyricsService,
    lyrics_enabled: bool,
    track: Option<TrackState>,
    published: Option<Published>,
    dirty: bool,
    paused_since: Option<Instant>,
}

impl Publisher {
    pub fn with_settings(settings: &Settings) -> Self {
        Self { lyrics_enabled: settings.media.lyrics, ..Self::default() }
    }

    pub fn apply_settings(&mut self, settings: &Settings) {
        if self.lyrics_enabled != settings.media.lyrics {
            self.lyrics_enabled = settings.media.lyrics;
            self.track = None;
        }
    }

    pub fn refresh(&mut self, ctx: &SourceContext, reading: Option<Reading>) {
        let Some(reading) = reading else {
            self.withdraw(ctx);
            return;
        };
        self.follow_track(&reading);
        self.collect_lyrics();
        playback_flag::set_playing(reading.playback.playing);
        if self.is_lingering_paused(&reading.playback) {
            self.withdraw_published(ctx);
        } else if self.needs_publish(&reading) {
            self.publish(ctx, &reading);
        }
    }

    pub fn withdraw(&mut self, ctx: &SourceContext) {
        playback_flag::set_playing(false);
        self.track = None;
        self.paused_since = None;
        self.withdraw_published(ctx);
    }

    fn withdraw_published(&mut self, ctx: &SourceContext) {
        if self.published.take().is_some() {
            ctx.remove(ACTIVITY_KEY);
        }
    }

    fn follow_track(&mut self, reading: &Reading) {
        let key = format!("{}\u{1f}{}", reading.source.id, reading.track.key());
        let unchanged = self.track.as_ref().is_some_and(|state| state.key == key);
        if unchanged {
            self.retry_missing_artwork(reading);
        } else {
            self.begin_track(key, reading);
        }
    }

    fn begin_track(&mut self, key: String, reading: &Reading) {
        let lyrics = self.lyrics_for(&key, &reading.track, reading.playback.duration_ms);
        let mut state = TrackState { key, art: None, art_attempts: 0, lyrics };
        load_artwork(&mut state, reading);
        self.track = Some(state);
        self.paused_since = None;
        self.dirty = true;
    }

    fn retry_missing_artwork(&mut self, reading: &Reading) {
        let Some(state) = &mut self.track else { return };
        if state.art.is_none() && state.art_attempts < MAX_ARTWORK_ATTEMPTS {
            load_artwork(state, reading);
            self.dirty |= state.art.is_some();
        }
    }

    fn lyrics_for(&mut self, key: &str, track: &Track, duration_ms: u64) -> Vec<LyricLine> {
        if !self.lyrics_enabled || track.title.is_empty() {
            return Vec::new();
        }
        let query = LyricsQuery {
            artist: track.artist.clone(),
            title: track.title.clone(),
            album: track.album.clone(),
            duration_secs: duration_ms / 1000,
        };
        self.lyrics.lookup(key, query).unwrap_or_default()
    }

    fn collect_lyrics(&mut self) {
        while let Some(ready) = self.lyrics.take_ready() {
            match &mut self.track {
                Some(state) if state.key == ready.track_key && self.lyrics_enabled => {
                    state.lyrics = ready.lines;
                    self.dirty = true;
                }
                _ => {}
            }
        }
    }

    fn is_lingering_paused(&mut self, playback: &Playback) -> bool {
        if playback.playing {
            self.paused_since = None;
            return false;
        }
        let since = *self.paused_since.get_or_insert_with(Instant::now);
        since.elapsed() > PAUSED_LINGER
    }

    fn needs_publish(&self, reading: &Reading) -> bool {
        match &self.published {
            Some(published) => self.dirty || published.differs_from(reading),
            None => true,
        }
    }

    fn publish(&mut self, ctx: &SourceContext, reading: &Reading) {
        let Some(state) = &self.track else { return };
        let frame = TrackFrame {
            title: &reading.track.title,
            artist: &reading.track.artist,
            album: &reading.track.album,
            art: state.art.as_deref(),
            lyrics: &state.lyrics,
            playback: &reading.playback,
            source: &reading.source,
            sources: &reading.sources,
        };
        ctx.upsert(build_activity(ctx, &frame));
        self.published = Some(Published::of(reading));
        self.dirty = false;
    }
}

fn load_artwork(state: &mut TrackState, reading: &Reading) {
    state.art_attempts += 1;
    state.art = reading.thumbnail.as_ref().and_then(artwork::read_data_url);
}
