use std::collections::HashMap;

use super::model::{Match, MatchState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighlightReason {
    ScoreChanged,
    FullTime,
}

pub struct Highlight {
    pub game: Match,
    pub reason: HighlightReason,
}

struct Seen {
    state: MatchState,
    scoreline: (u32, u32),
}

impl Seen {
    fn of(game: &Match) -> Self {
        Self { state: game.state, scoreline: game.scoreline() }
    }

    fn reason_for_change(&self, game: &Match) -> Option<HighlightReason> {
        let was_playing = self.state == MatchState::Live;
        if (was_playing || game.is_live()) && self.scoreline != game.scoreline() {
            return Some(HighlightReason::ScoreChanged);
        }
        let finished_now = was_playing && game.state == MatchState::Finished;
        finished_now.then_some(HighlightReason::FullTime)
    }
}

#[derive(Default)]
pub struct ScoreTracker {
    seen: HashMap<String, Seen>,
}

impl ScoreTracker {
    pub fn observe(&mut self, games: &[Match]) -> Vec<Highlight> {
        let highlights = games.iter().filter_map(|game| self.highlight_for(game)).collect();
        self.seen = games.iter().map(|game| (game.id.clone(), Seen::of(game))).collect();
        highlights
    }

    fn highlight_for(&self, game: &Match) -> Option<Highlight> {
        let reason = self.seen.get(&game.id)?.reason_for_change(game)?;
        Some(Highlight { game: game.clone(), reason })
    }
}
