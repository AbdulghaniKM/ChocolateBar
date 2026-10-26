use std::fs;
use std::path::{Path, PathBuf};

use keyvalues_parser::{Obj, Value};
use windows::core::w;

use super::model::{GameDownload, GameInstall, Launcher, LibrarySnapshot};
use super::registry::read_current_user_string;

const UPDATE_RUNNING: u64 = 0x100;
const UPDATE_PAUSED: u64 = 0x200;
const DOWNLOADING: u64 = 0x10_0000;
const STAGING: u64 = 0x20_0000;
const ACTIVE_TRANSFER_FLAGS: u64 = UPDATE_RUNNING | DOWNLOADING | STAGING;

pub fn scan() -> LibrarySnapshot {
    let mut snapshot = LibrarySnapshot::default();
    let Some(root) = steam_root() else {
        return snapshot;
    };
    for library in library_folders(&root) {
        scan_library(&library, &mut snapshot);
    }
    snapshot
}

fn steam_root() -> Option<PathBuf> {
    read_current_user_string(w!("Software\\Valve\\Steam"), w!("SteamPath")).map(PathBuf::from)
}

fn library_folders(root: &Path) -> Vec<PathBuf> {
    let mut folders = vec![root.to_path_buf()];
    let listing = root.join("steamapps").join("libraryfolders.vdf");
    let Ok(text) = fs::read_to_string(listing) else {
        return folders;
    };
    let Ok(parsed) = keyvalues_parser::parse(&text) else {
        return folders;
    };
    let vdf = parsed.into_vdf();
    let entries = vdf.value.get_obj().into_iter().flat_map(|obj| obj.values().flatten());
    for path in entries.filter_map(|entry| entry.get_obj()).filter_map(|obj| text_of(obj, "path")) {
        let folder = PathBuf::from(path);
        if !folders.contains(&folder) {
            folders.push(folder);
        }
    }
    folders
}

fn scan_library(library: &Path, snapshot: &mut LibrarySnapshot) {
    let steamapps = library.join("steamapps");
    let Ok(entries) = fs::read_dir(&steamapps) else {
        return;
    };
    for path in entries.flatten().map(|entry| entry.path()).filter(|path| is_app_manifest(path)) {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        if let Some(manifest) = AppManifest::parse(&text) {
            manifest.record_into(&steamapps, snapshot);
        }
    }
}

fn is_app_manifest(path: &Path) -> bool {
    let name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();
    name.starts_with("appmanifest_") && name.ends_with(".acf")
}

fn text_of<'a>(obj: &'a Obj<'_>, key: &str) -> Option<&'a str> {
    obj.get(key)?.first().and_then(Value::get_str)
}

fn number_of(obj: &Obj<'_>, key: &str) -> Option<u64> {
    text_of(obj, key)?.trim().parse().ok()
}

struct AppManifest {
    app_id: String,
    name: String,
    install_dir: String,
    state_flags: u64,
    bytes_downloaded: Option<u64>,
    bytes_to_download: Option<u64>,
}

impl AppManifest {
    fn parse(text: &str) -> Option<Self> {
        let vdf = keyvalues_parser::parse(text).ok()?.into_vdf();
        let state = vdf.value.get_obj()?;
        Some(Self {
            app_id: text_of(state, "appid")?.to_string(),
            name: text_of(state, "name")?.to_string(),
            install_dir: text_of(state, "installdir")?.to_string(),
            state_flags: number_of(state, "StateFlags").unwrap_or_default(),
            bytes_downloaded: number_of(state, "BytesDownloaded"),
            bytes_to_download: number_of(state, "BytesToDownload"),
        })
    }

    fn record_into(self, steamapps: &Path, snapshot: &mut LibrarySnapshot) {
        let id = format!("steam-{}", self.app_id);
        snapshot.installs.push(GameInstall {
            id: id.clone(),
            name: self.name.clone(),
            dir: steamapps.join("common").join(&self.install_dir),
        });
        if let Some(paused) = self.transfer_in_progress() {
            snapshot.downloads.push(GameDownload {
                id,
                name: self.name,
                launcher: Launcher::Steam,
                paused,
                downloaded_bytes: self.bytes_downloaded,
                total_bytes: self.bytes_to_download,
            });
        }
    }

    fn transfer_in_progress(&self) -> Option<bool> {
        let transferring = self.state_flags & ACTIVE_TRANSFER_FLAGS != 0;
        let paused = self.state_flags & UPDATE_PAUSED != 0;
        let pending_bytes = self.bytes_to_download.is_some_and(|bytes| bytes > 0);
        (transferring || (paused && pending_bytes)).then_some(paused && !transferring)
    }
}
