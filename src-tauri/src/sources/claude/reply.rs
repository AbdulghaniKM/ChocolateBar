use crate::core::clipboard;
use crate::sources::launcher::desktop::{self, WindowId};

const EDITORS: [&str; 6] = ["code.exe", "code - insiders.exe", "cursor.exe", "windsurf.exe", "codium.exe", "zed.exe"];
const TERMINALS: [&str; 6] = ["windowsterminal.exe", "wt.exe", "pwsh.exe", "powershell.exe", "cmd.exe", "wezterm-gui.exe"];
const EDITOR_SCORE: u32 = 4;
const TERMINAL_SCORE: u32 = 2;
const PROJECT_SCORE: u32 = 3;

pub struct Candidate {
    pub id: WindowId,
    pub title: String,
    pub process: String,
}

/// Puts the reply on the clipboard and brings forward the editor or terminal running the session,
/// since Claude Code accepts input only from its own window.
pub fn hand_off(text: &str, project: Option<&str>) -> Result<Option<String>, String> {
    if !clipboard::write_text(text) {
        return Err("Couldn't put the message on the clipboard".into());
    }
    let candidates = desktop::app_windows()
        .into_iter()
        .map(|id| Candidate { id, title: desktop::title_of(id), process: desktop::process_name_of(id) })
        .collect::<Vec<_>>();
    let Some(target) = best_window(&candidates, project) else {
        return Ok(None);
    };
    desktop::bring_to_front(target.id);
    Ok(Some(target.title.clone()))
}

pub fn best_window<'a>(candidates: &'a [Candidate], project: Option<&str>) -> Option<&'a Candidate> {
    let mut best: Option<(&Candidate, u32)> = None;
    for candidate in candidates {
        let score = score(candidate, project);
        if score > best.map_or(0, |(_, top)| top) {
            best = Some((candidate, score));
        }
    }
    best.filter(|(_, score)| *score >= PROJECT_SCORE).map(|(candidate, _)| candidate)
}

fn score(candidate: &Candidate, project: Option<&str>) -> u32 {
    let kind = if EDITORS.contains(&candidate.process.as_str()) {
        EDITOR_SCORE
    } else if TERMINALS.contains(&candidate.process.as_str()) {
        TERMINAL_SCORE
    } else {
        return 0;
    };
    let names_project = project
        .filter(|name| !name.is_empty())
        .is_some_and(|name| candidate.title.to_lowercase().contains(&name.to_lowercase()));
    kind + if names_project { PROJECT_SCORE } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn window(id: WindowId, title: &str, process: &str) -> Candidate {
        Candidate { id, title: title.into(), process: process.into() }
    }

    #[test]
    fn prefers_the_editor_showing_the_session_project() {
        let candidates = [
            window(1, "notes.md - Other - Visual Studio Code", "code.exe"),
            window(2, "lib.rs - ChocolateBar - Visual Studio Code", "code.exe"),
            window(3, "ChocolateBar", "explorer.exe"),
        ];
        assert_eq!(best_window(&candidates, Some("ChocolateBar")).map(|found| found.id), Some(2));
    }

    #[test]
    fn falls_back_to_any_editor_but_never_to_unrelated_apps() {
        let candidates = [window(1, "Spotify", "spotify.exe"), window(2, "main.rs - Visual Studio Code", "code.exe")];
        assert_eq!(best_window(&candidates, Some("Elsewhere")).map(|found| found.id), Some(2));
        assert!(best_window(&[window(1, "ChocolateBar", "spotify.exe")], Some("ChocolateBar")).is_none());
    }

    #[test]
    fn a_terminal_needs_the_project_in_its_title() {
        let terminal = [window(1, "claude - ChocolateBar", "windowsterminal.exe")];
        assert_eq!(best_window(&terminal, Some("ChocolateBar")).map(|found| found.id), Some(1));
        assert!(best_window(&[window(1, "PowerShell", "windowsterminal.exe")], Some("ChocolateBar")).is_none());
    }
}
