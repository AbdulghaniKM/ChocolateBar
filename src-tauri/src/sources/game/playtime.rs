use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

use crate::core::activity::now_ms;

use super::model::GameInstall;

const STORE_FILE: &str = "playtime.json";
const TOTALS_KEY: &str = "totals";
const FLUSH_INTERVAL: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaytimeEntry {
    pub name: String,
    pub seconds: u64,
}

struct PlaytimeLedger {
    app: AppHandle,
    totals: BTreeMap<String, PlaytimeEntry>,
    dirty: bool,
}

impl PlaytimeLedger {
    fn load(app: &AppHandle) -> Self {
        let saved = app
            .store(STORE_FILE)
            .ok()
            .and_then(|store| store.get(TOTALS_KEY))
            .and_then(|value| serde_json::from_value(value).ok());
        Self { app: app.clone(), totals: saved.unwrap_or_default(), dirty: false }
    }

    fn add(&mut self, install: &GameInstall, played: Duration) {
        let entry = self.totals.entry(install.id.clone()).or_default();
        entry.name.clone_from(&install.name);
        entry.seconds += played.as_secs();
        self.dirty = true;
    }

    fn seconds_for(&self, id: &str) -> u64 {
        self.totals.get(id).map_or(0, |entry| entry.seconds)
    }

    fn flush(&mut self) {
        if !self.dirty {
            return;
        }
        match self.write_store() {
            Ok(()) => self.dirty = false,
            Err(error) => log::warn!("game: saving playtime failed: {error}"),
        }
    }

    fn write_store(&self) -> Result<(), String> {
        let store = self.app.store(STORE_FILE).map_err(|error| error.to_string())?;
        let value = serde_json::to_value(&self.totals).map_err(|error| error.to_string())?;
        store.set(TOTALS_KEY, value);
        store.save().map_err(|error| error.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct PlayingSession {
    pub id: String,
    pub name: String,
    pub started_at_ms: u64,
    pub elapsed: Duration,
}

pub struct PlaytimeTracker {
    ledger: PlaytimeLedger,
    session: Option<PlayingSession>,
    last_tick: Instant,
    last_flush: Instant,
}

impl PlaytimeTracker {
    pub fn load(app: &AppHandle) -> Self {
        Self {
            ledger: PlaytimeLedger::load(app),
            session: None,
            last_tick: Instant::now(),
            last_flush: Instant::now(),
        }
    }

    pub fn session(&self) -> Option<&PlayingSession> {
        self.session.as_ref()
    }

    pub fn total_seconds_for(&self, id: &str) -> u64 {
        self.ledger.seconds_for(id)
    }

    pub fn totals(&self) -> Vec<PlaytimeEntry> {
        let mut entries: Vec<PlaytimeEntry> = self.ledger.totals.values().cloned().collect();
        entries.sort_by_key(|entry| std::cmp::Reverse(entry.seconds));
        entries
    }

    pub fn observe(&mut self, running: Option<&GameInstall>) {
        let since_last_tick = self.last_tick.elapsed();
        self.last_tick = Instant::now();
        match running {
            Some(install) if self.is_playing(install) => self.accumulate(install, since_last_tick),
            Some(install) => self.begin_session(install),
            None => self.end_session(),
        }
        if self.last_flush.elapsed() >= FLUSH_INTERVAL {
            self.flush();
        }
    }

    fn is_playing(&self, install: &GameInstall) -> bool {
        self.session.as_ref().is_some_and(|session| session.id == install.id)
    }

    fn accumulate(&mut self, install: &GameInstall, played: Duration) {
        self.ledger.add(install, played);
        if let Some(session) = self.session.as_mut() {
            session.elapsed += played;
        }
    }

    fn begin_session(&mut self, install: &GameInstall) {
        self.end_session();
        self.session = Some(PlayingSession {
            id: install.id.clone(),
            name: install.name.clone(),
            started_at_ms: now_ms(),
            elapsed: Duration::ZERO,
        });
    }

    fn end_session(&mut self) {
        if self.session.take().is_some() {
            self.flush();
        }
    }

    fn flush(&mut self) {
        self.ledger.flush();
        self.last_flush = Instant::now();
    }
}
