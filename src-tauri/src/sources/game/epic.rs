use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::model::{GameDownload, GameInstall, Launcher, LibrarySnapshot};

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "PascalCase")]
struct ItemDto {
    display_name: Option<String>,
    app_name: Option<String>,
    install_location: Option<String>,
    #[serde(rename = "bIsIncompleteInstall")]
    is_incomplete_install: Option<bool>,
    install_size: Option<u64>,
}

pub fn scan() -> LibrarySnapshot {
    let mut snapshot = LibrarySnapshot::default();
    let Some(manifests) = manifests_dir() else {
        return snapshot;
    };
    let Ok(entries) = fs::read_dir(manifests) else {
        return snapshot;
    };
    for path in entries.flatten().map(|entry| entry.path()).filter(|path| is_item_file(path)) {
        if let Some(item) = read_item(&path) {
            record(item, &mut snapshot);
        }
    }
    snapshot
}

fn manifests_dir() -> Option<PathBuf> {
    let program_data = std::env::var_os("ProgramData")?;
    Some(PathBuf::from(program_data).join("Epic/EpicGamesLauncher/Data/Manifests"))
}

fn is_item_file(path: &Path) -> bool {
    path.extension().and_then(|extension| extension.to_str()) == Some("item")
}

fn read_item(path: &Path) -> Option<ItemDto> {
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

fn record(item: ItemDto, snapshot: &mut LibrarySnapshot) {
    let (Some(app_name), Some(location)) = (item.app_name, item.install_location) else {
        return;
    };
    let name = item.display_name.unwrap_or_else(|| app_name.clone());
    let id = format!("epic-{app_name}");
    snapshot.installs.push(GameInstall { id: id.clone(), name: name.clone(), dir: PathBuf::from(location) });
    if item.is_incomplete_install == Some(true) {
        snapshot.downloads.push(GameDownload {
            id,
            name,
            launcher: Launcher::Epic,
            paused: false,
            downloaded_bytes: None,
            total_bytes: item.install_size,
        });
    }
}
