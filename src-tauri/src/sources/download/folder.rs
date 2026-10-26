use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

const PARTIAL_EXTENSIONS: [&str; 3] = ["crdownload", "part", "partial"];
const FRESH_PARTIAL_WINDOW: Duration = Duration::from_secs(30);
const UNCONFIRMED_PREFIX: &str = "Unconfirmed ";

pub fn downloads_folder() -> Option<PathBuf> {
    dirs::download_dir()
}

pub fn is_partial_download(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| PARTIAL_EXTENSIONS.iter().any(|known| extension.eq_ignore_ascii_case(known)))
}

pub fn completed_path_of(partial: &Path) -> PathBuf {
    partial.with_extension("")
}

pub fn display_name_of(partial: &Path) -> String {
    let name = completed_path_of(partial)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    if name.is_empty() || name.starts_with(UNCONFIRMED_PREFIX) {
        "Starting download".into()
    } else {
        name
    }
}

pub fn scan_fresh_partial_downloads(folder: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(folder) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| is_partial_download(path) && was_modified_recently(path))
        .collect()
}

fn was_modified_recently(path: &Path) -> bool {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| SystemTime::now().duration_since(modified).ok())
        .is_some_and(|age| age <= FRESH_PARTIAL_WINDOW)
}
