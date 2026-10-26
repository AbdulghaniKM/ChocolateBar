use std::path::{Path, PathBuf};

use crossbeam_channel::{unbounded, Receiver};
use notify::{recommended_watcher, Event, RecommendedWatcher, RecursiveMode, Watcher};

use super::folder::is_partial_download;

pub struct FolderWatch {
    _watcher: RecommendedWatcher,
    touched_partials: Receiver<PathBuf>,
}

impl FolderWatch {
    pub fn start(folder: &Path) -> notify::Result<Self> {
        let (sender, touched_partials) = unbounded();
        let mut watcher = recommended_watcher(move |event: notify::Result<Event>| match event {
            Ok(event) => {
                for path in event.paths.into_iter().filter(|path| is_partial_download(path)) {
                    let _ = sender.send(path);
                }
            }
            Err(error) => log::warn!("download: watcher event failed: {error}"),
        })?;
        watcher.watch(folder, RecursiveMode::NonRecursive)?;
        Ok(Self { _watcher: watcher, touched_partials })
    }

    pub fn drain_touched_partials(&self) -> Vec<PathBuf> {
        self.touched_partials.try_iter().collect()
    }
}
