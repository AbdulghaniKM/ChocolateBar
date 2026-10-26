use std::collections::HashSet;

use serde_json::Value;

use crate::core::activity::SourceCommand;
use crate::core::source::SourceContext;

use super::model::{GameDownload, LibrarySnapshot};
use super::presenter::{download_activity, download_key, playing_activity, playing_key};
use super::process_scan::RunningGameDetector;
use super::playtime::PlaytimeTracker;
use super::{epic, steam};

const PLAYTIME_COMMAND: &str = "playtime";

pub struct GameService {
    ctx: SourceContext,
    detector: RunningGameDetector,
    tracker: PlaytimeTracker,
    published_downloads: HashSet<String>,
    playing_published: bool,
}

impl GameService {
    pub fn new(ctx: SourceContext) -> Self {
        let tracker = PlaytimeTracker::load(&ctx.app);
        Self {
            ctx,
            detector: RunningGameDetector::new(),
            tracker,
            published_downloads: HashSet::new(),
            playing_published: false,
        }
    }

    pub fn scan(&mut self) {
        let snapshot = scan_libraries();
        self.sync_downloads(&snapshot.downloads);
        let running = self.detector.find_running(&snapshot.installs);
        self.tracker.observe(running);
        self.sync_playing();
    }

    pub fn answer(&self, command: SourceCommand) {
        if command.command != PLAYTIME_COMMAND {
            return command.respond_err("command not supported");
        }
        let totals = serde_json::to_value(self.tracker.totals()).unwrap_or(Value::Null);
        command.respond_ok(totals);
    }

    fn sync_downloads(&mut self, downloads: &[GameDownload]) {
        let current: HashSet<String> = downloads.iter().map(|download| download_key(&download.id)).collect();
        for finished in self.published_downloads.difference(&current) {
            self.ctx.remove(finished);
        }
        for download in downloads {
            self.ctx.upsert(download_activity(download));
        }
        self.published_downloads = current;
    }

    fn sync_playing(&mut self) {
        match self.tracker.session() {
            Some(session) => {
                let total = self.tracker.total_seconds_for(&session.id);
                self.ctx.upsert(playing_activity(session, total));
                self.playing_published = true;
            }
            None if self.playing_published => {
                self.ctx.remove(playing_key());
                self.playing_published = false;
            }
            None => {}
        }
    }
}

fn scan_libraries() -> LibrarySnapshot {
    let mut snapshot = steam::scan();
    snapshot.absorb(epic::scan());
    snapshot
}
