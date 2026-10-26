use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::folder::{completed_path_of, display_name_of};

const SPEED_SMOOTHING: f64 = 0.5;
const SPEED_FLOOR_BYTES_PER_SEC: f64 = 1.0;

#[derive(Debug, Clone)]
pub struct DownloadSnapshot {
    pub key: String,
    pub file_name: String,
    pub bytes_done: u64,
    pub bytes_per_second: f64,
    pub elapsed: Duration,
    pub idle_for: Duration,
}

pub enum DownloadUpdate {
    InProgress(DownloadSnapshot),
    Completed(DownloadSnapshot),
    Vanished { key: String },
}

struct TrackedDownload {
    key: String,
    started_at: Instant,
    last_size: u64,
    last_sample_at: Instant,
    last_growth_at: Instant,
    bytes_per_second: f64,
}

impl TrackedDownload {
    fn begin(path: &Path, now: Instant) -> Self {
        let size = fs::metadata(path).map(|metadata| metadata.len()).unwrap_or(0);
        Self {
            key: key_for(path),
            started_at: now,
            last_size: size,
            last_sample_at: now,
            last_growth_at: now,
            bytes_per_second: 0.0,
        }
    }

    fn record_size(&mut self, size: u64, now: Instant) {
        let seconds = now.duration_since(self.last_sample_at).as_secs_f64().max(f64::EPSILON);
        let instantaneous = size.saturating_sub(self.last_size) as f64 / seconds;
        let smoothed = SPEED_SMOOTHING * instantaneous + (1.0 - SPEED_SMOOTHING) * self.bytes_per_second;
        self.bytes_per_second = if smoothed < SPEED_FLOOR_BYTES_PER_SEC { 0.0 } else { smoothed };
        if size > self.last_size {
            self.last_growth_at = now;
        }
        self.last_size = size;
        self.last_sample_at = now;
    }

    fn snapshot(&self, path: &Path, now: Instant) -> DownloadSnapshot {
        DownloadSnapshot {
            key: self.key.clone(),
            file_name: display_name_of(path),
            bytes_done: self.last_size,
            bytes_per_second: self.bytes_per_second,
            elapsed: now.duration_since(self.started_at),
            idle_for: now.duration_since(self.last_growth_at),
        }
    }
}

#[derive(Default)]
pub struct DownloadTracker {
    tracked: HashMap<PathBuf, TrackedDownload>,
}

impl DownloadTracker {
    pub fn track(&mut self, partial: PathBuf, now: Instant) {
        if partial.exists() && !self.tracked.contains_key(&partial) {
            let download = TrackedDownload::begin(&partial, now);
            self.tracked.insert(partial, download);
        }
    }

    pub fn sample(&mut self, now: Instant) -> Vec<DownloadUpdate> {
        let paths: Vec<PathBuf> = self.tracked.keys().cloned().collect();
        paths.into_iter().filter_map(|path| self.sample_one(path, now)).collect()
    }

    fn sample_one(&mut self, path: PathBuf, now: Instant) -> Option<DownloadUpdate> {
        match fs::metadata(&path) {
            Ok(metadata) => {
                let download = self.tracked.get_mut(&path)?;
                download.record_size(metadata.len(), now);
                Some(DownloadUpdate::InProgress(download.snapshot(&path, now)))
            }
            Err(_) => self.finish(path, now),
        }
    }

    fn finish(&mut self, path: PathBuf, now: Instant) -> Option<DownloadUpdate> {
        let download = self.tracked.remove(&path)?;
        let completed_path = completed_path_of(&path);
        let Ok(metadata) = fs::metadata(&completed_path) else {
            return Some(DownloadUpdate::Vanished { key: download.key });
        };
        let mut snapshot = download.snapshot(&path, now);
        snapshot.bytes_done = metadata.len();
        snapshot.bytes_per_second = 0.0;
        Some(DownloadUpdate::Completed(snapshot))
    }
}

fn key_for(path: &Path) -> String {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    format!("file-{:016x}", hasher.finish())
}
