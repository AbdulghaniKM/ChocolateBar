mod folder;
mod tracker;
pub(crate) mod units;
mod view;
mod watch;

use std::path::Path;
use std::time::{Duration, Instant};

use crate::core::source::{ActivitySource, SourceContext, SourceEvent};

use tracker::{DownloadTracker, DownloadUpdate};
use watch::FolderWatch;

const WAKE_INTERVAL: Duration = Duration::from_millis(250);
const SAMPLE_INTERVAL: Duration = Duration::from_secs(1);
const HIDE_AFTER_IDLE: Duration = Duration::from_secs(120);

#[derive(Default)]
pub struct DownloadSource;

impl ActivitySource for DownloadSource {
    fn name(&self) -> &'static str {
        "download"
    }

    fn start(self: Box<Self>, ctx: SourceContext) {
        let Some(folder) = folder::downloads_folder() else {
            log::warn!("download: no Downloads folder found, source idle");
            ctx.idle_until_shutdown();
            return;
        };
        let watch = FolderWatch::start(&folder)
            .inspect_err(|error| log::warn!("download: folder watch unavailable, scanning instead: {error}"))
            .ok();
        watch_downloads(&ctx, &folder, watch.as_ref());
    }
}

fn watch_downloads(ctx: &SourceContext, folder: &Path, watch: Option<&FolderWatch>) {
    let mut tracker = DownloadTracker::default();
    let mut last_sample = Instant::now();
    track_all(&mut tracker, folder::scan_fresh_partial_downloads(folder));
    while ctx.is_running() {
        match ctx.next_event(WAKE_INTERVAL) {
            SourceEvent::Shutdown => break,
            SourceEvent::Command(command) => command.respond_err("command not supported"),
            SourceEvent::Action(_) | SourceEvent::Idle => {}
        }
        if let Some(watch) = watch {
            track_all(&mut tracker, watch.drain_touched_partials());
        }
        if last_sample.elapsed() >= SAMPLE_INTERVAL {
            last_sample = Instant::now();
            if watch.is_none() {
                track_all(&mut tracker, folder::scan_fresh_partial_downloads(folder));
            }
            publish(ctx, tracker.sample(Instant::now()));
        }
    }
}

fn track_all(tracker: &mut DownloadTracker, partials: Vec<std::path::PathBuf>) {
    let now = Instant::now();
    for partial in partials {
        tracker.track(partial, now);
    }
}

fn publish(ctx: &SourceContext, updates: Vec<DownloadUpdate>) {
    for update in updates {
        match update {
            DownloadUpdate::InProgress(snapshot) if snapshot.idle_for >= HIDE_AFTER_IDLE => ctx.remove(&snapshot.key),
            DownloadUpdate::InProgress(snapshot) => ctx.upsert(view::progress_activity(ctx, &snapshot)),
            DownloadUpdate::Completed(snapshot) => ctx.upsert(view::completed_activity(ctx, &snapshot)),
            DownloadUpdate::Vanished { key } => ctx.remove(&key),
        }
    }
}
