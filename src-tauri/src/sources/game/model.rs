use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct GameInstall {
    pub id: String,
    pub name: String,
    pub dir: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Launcher {
    Steam,
    Epic,
}

impl Launcher {
    pub fn label(self) -> &'static str {
        match self {
            Self::Steam => "Steam",
            Self::Epic => "Epic Games",
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameDownload {
    pub id: String,
    pub name: String,
    pub launcher: Launcher,
    pub paused: bool,
    pub downloaded_bytes: Option<u64>,
    pub total_bytes: Option<u64>,
}

impl GameDownload {
    pub fn fraction(&self) -> Option<f32> {
        let total = self.total_bytes.filter(|&bytes| bytes > 0)?;
        Some((self.downloaded_bytes? as f64 / total as f64).clamp(0.0, 1.0) as f32)
    }
}

#[derive(Debug, Default)]
pub struct LibrarySnapshot {
    pub installs: Vec<GameInstall>,
    pub downloads: Vec<GameDownload>,
}

impl LibrarySnapshot {
    pub fn absorb(&mut self, other: LibrarySnapshot) {
        self.installs.extend(other.installs);
        self.downloads.extend(other.downloads);
    }
}
