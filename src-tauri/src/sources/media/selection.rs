use std::collections::HashSet;

use super::session::SourceSummary;

/// Decides which media session the island follows: whatever most recently started playing,
/// unless the user picked one, falling back to the session Windows itself calls current.
#[derive(Default)]
pub struct Selection {
    active: Option<String>,
    was_playing: HashSet<String>,
}

impl Selection {
    pub fn active(&self) -> Option<&str> {
        self.active.as_deref()
    }

    pub fn pick(&mut self, id: &str) {
        self.active = Some(id.to_string());
    }

    pub fn choose(&mut self, sources: &[SourceSummary], current: Option<&str>) -> Option<String> {
        let playing: HashSet<String> =
            sources.iter().filter(|source| source.playing).map(|source| source.id.clone()).collect();
        let newly_playing = sources.iter().find(|source| source.playing && !self.was_playing.contains(&source.id));
        let active_exists = self.active.as_ref().is_some_and(|id| sources.iter().any(|source| &source.id == id));
        if let Some(started) = newly_playing {
            self.active = Some(started.id.clone());
        } else if !active_exists {
            self.active = fallback(sources, current);
        }
        self.was_playing = playing;
        self.active.clone()
    }
}

fn fallback(sources: &[SourceSummary], current: Option<&str>) -> Option<String> {
    let is_current = |source: &&SourceSummary| Some(source.id.as_str()) == current;
    sources
        .iter()
        .filter(|source| source.playing)
        .find(is_current)
        .or_else(|| sources.iter().find(|source| source.playing))
        .or_else(|| sources.iter().find(is_current))
        .or_else(|| sources.first())
        .map(|source| source.id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(id: &str, playing: bool) -> SourceSummary {
        SourceSummary { id: id.into(), app: id.into(), kind: "app", playing }
    }

    #[test]
    fn prefers_a_playing_browser_over_a_paused_current_session() {
        let mut selection = Selection::default();
        let sources = [source("Spotify.exe", false), source("Chrome", true)];
        assert_eq!(selection.choose(&sources, Some("Spotify.exe")).as_deref(), Some("Chrome"));
    }

    #[test]
    fn follows_the_session_that_just_started_playing() {
        let mut selection = Selection::default();
        selection.choose(&[source("Spotify.exe", true), source("Brave", false)], None);
        let chosen = selection.choose(&[source("Spotify.exe", true), source("Brave", true)], None);
        assert_eq!(chosen.as_deref(), Some("Brave"));
    }

    #[test]
    fn keeps_a_manual_pick_until_something_new_starts() {
        let mut selection = Selection::default();
        let sources = [source("Spotify.exe", true), source("Brave", false)];
        selection.choose(&sources, None);
        selection.pick("Brave");
        assert_eq!(selection.choose(&sources, None).as_deref(), Some("Brave"));
    }

    #[test]
    fn falls_back_when_the_active_session_closes() {
        let mut selection = Selection::default();
        selection.pick("Brave");
        assert_eq!(selection.choose(&[source("Spotify.exe", false)], None).as_deref(), Some("Spotify.exe"));
    }
}
