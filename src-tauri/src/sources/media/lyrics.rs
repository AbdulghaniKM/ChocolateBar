use std::collections::VecDeque;
use std::time::Duration;

use crossbeam_channel::{unbounded, Receiver, Sender};
use reqwest::Client;
use serde::Deserialize;

use super::lrc::{parse_lrc, LyricLine};

const LRCLIB_API: &str = "https://lrclib.net/api";
const USER_AGENT: &str = concat!("ChocolateBar/", env!("CARGO_PKG_VERSION"), " (Windows Dynamic Island bar)");
const REQUEST_TIMEOUT: Duration = Duration::from_secs(8);
const CACHE_CAPACITY: usize = 24;

#[derive(Clone)]
pub struct LyricsQuery {
    pub artist: String,
    pub title: String,
    pub album: String,
    pub duration_secs: u64,
}

pub struct LyricsResult {
    pub track_key: String,
    pub lines: Vec<LyricLine>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LrclibRecord {
    synced_lyrics: Option<String>,
    duration: Option<f64>,
}

pub struct LyricsService {
    client: Client,
    cache: VecDeque<(String, Vec<LyricLine>)>,
    requested_key: Option<String>,
    results_sender: Sender<LyricsResult>,
    results_receiver: Receiver<LyricsResult>,
}

impl Default for LyricsService {
    fn default() -> Self {
        let client = Client::builder().user_agent(USER_AGENT).timeout(REQUEST_TIMEOUT).build();
        let (results_sender, results_receiver) = unbounded();
        Self {
            client: client.unwrap_or_default(),
            cache: VecDeque::new(),
            requested_key: None,
            results_sender,
            results_receiver,
        }
    }
}

impl LyricsService {
    pub fn lookup(&mut self, track_key: &str, query: LyricsQuery) -> Option<Vec<LyricLine>> {
        if let Some(cached) = self.cached(track_key) {
            return Some(cached);
        }
        if self.requested_key.as_deref() != Some(track_key) {
            self.requested_key = Some(track_key.to_string());
            self.spawn_fetch(track_key.to_string(), query);
        }
        None
    }

    pub fn take_ready(&mut self) -> Option<LyricsResult> {
        let result = self.results_receiver.try_recv().ok()?;
        self.remember(result.track_key.clone(), result.lines.clone());
        Some(result)
    }

    fn cached(&self, track_key: &str) -> Option<Vec<LyricLine>> {
        self.cache.iter().find(|(key, _)| key == track_key).map(|(_, lines)| lines.clone())
    }

    fn remember(&mut self, track_key: String, lines: Vec<LyricLine>) {
        if self.cache.len() >= CACHE_CAPACITY {
            self.cache.pop_front();
        }
        self.cache.push_back((track_key, lines));
    }

    fn spawn_fetch(&self, track_key: String, query: LyricsQuery) {
        let client = self.client.clone();
        let sender = self.results_sender.clone();
        tauri::async_runtime::spawn(async move {
            let lines = fetch_synced_lines(&client, &query).await;
            let _ = sender.send(LyricsResult { track_key, lines });
        });
    }
}

async fn fetch_synced_lines(client: &Client, query: &LyricsQuery) -> Vec<LyricLine> {
    let synced = match fetch_exact(client, query).await {
        Some(synced) => Some(synced),
        None => fetch_from_search(client, query).await,
    };
    synced.map(|text| parse_lrc(&text)).unwrap_or_default()
}

async fn fetch_exact(client: &Client, query: &LyricsQuery) -> Option<String> {
    let mut parameters = vec![("artist_name", query.artist.clone()), ("track_name", query.title.clone())];
    if !query.album.is_empty() {
        parameters.push(("album_name", query.album.clone()));
    }
    if query.duration_secs > 0 {
        parameters.push(("duration", query.duration_secs.to_string()));
    }
    let response = client.get(format!("{LRCLIB_API}/get")).query(&parameters).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    non_empty_synced(response.json::<LrclibRecord>().await.ok()?)
}

async fn fetch_from_search(client: &Client, query: &LyricsQuery) -> Option<String> {
    let parameters = [("artist_name", query.artist.as_str()), ("track_name", query.title.as_str())];
    let response = client.get(format!("{LRCLIB_API}/search")).query(&parameters).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    let records = response.json::<Vec<LrclibRecord>>().await.ok()?;
    closest_in_duration(records, query.duration_secs).and_then(non_empty_synced)
}

fn closest_in_duration(records: Vec<LrclibRecord>, duration_secs: u64) -> Option<LrclibRecord> {
    let target = duration_secs as f64;
    records
        .into_iter()
        .filter(non_empty_synced_ref)
        .min_by(|left, right| duration_gap(left, target).total_cmp(&duration_gap(right, target)))
}

fn duration_gap(record: &LrclibRecord, target: f64) -> f64 {
    record.duration.map_or(f64::MAX, |duration| (duration - target).abs())
}

fn non_empty_synced_ref(record: &LrclibRecord) -> bool {
    record.synced_lyrics.as_deref().is_some_and(|synced| !synced.trim().is_empty())
}

fn non_empty_synced(record: LrclibRecord) -> Option<String> {
    record.synced_lyrics.filter(|synced| !synced.trim().is_empty())
}
