use std::path::Path;

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

use super::model::GameInstall;

pub struct RunningGameDetector {
    system: System,
}

impl RunningGameDetector {
    pub fn new() -> Self {
        Self { system: System::new() }
    }

    pub fn find_running<'a>(&mut self, installs: &'a [GameInstall]) -> Option<&'a GameInstall> {
        self.refresh_executables();
        let install_roots: Vec<(String, &GameInstall)> =
            installs.iter().map(|install| (directory_prefix(&install.dir), install)).collect();
        self.system
            .processes()
            .values()
            .filter_map(|process| process.exe())
            .map(normalized)
            .filter_map(|exe| owner_of(&exe, &install_roots))
            .min_by(|a, b| a.id.cmp(&b.id))
    }

    fn refresh_executables(&mut self) {
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_exe(UpdateKind::OnlyIfNotSet),
        );
    }
}

fn owner_of<'a>(exe: &str, install_roots: &[(String, &'a GameInstall)]) -> Option<&'a GameInstall> {
    install_roots.iter().find(|(root, _)| exe.starts_with(root.as_str())).map(|(_, install)| *install)
}

fn normalized(path: &Path) -> String {
    path.to_string_lossy().replace('/', "\\").to_lowercase()
}

fn directory_prefix(dir: &Path) -> String {
    format!("{}\\", normalized(dir).trim_end_matches('\\'))
}
