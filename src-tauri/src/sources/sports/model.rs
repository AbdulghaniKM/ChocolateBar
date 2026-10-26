use serde::Serialize;

use crate::core::settings::TeamRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchState {
    Upcoming,
    Live,
    Finished,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Side {
    pub id: String,
    pub name: String,
    pub abbreviation: String,
    pub logo: Option<String>,
    pub score: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Match {
    pub id: String,
    pub sport: String,
    pub league: String,
    pub state: MatchState,
    pub status: String,
    pub home: Side,
    pub away: Side,
}

impl Match {
    pub fn scoreline(&self) -> (u32, u32) {
        (self.home.score, self.away.score)
    }

    pub fn is_live(&self) -> bool {
        self.state == MatchState::Live
    }

    fn involves(&self, team: &TeamRef) -> bool {
        team.league == self.league && (self.home.id == team.id || self.away.id == team.id)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamEntry {
    pub sport: String,
    pub league: String,
    pub id: String,
    pub name: String,
    pub logo: Option<String>,
}

pub fn keep_favorites(matches: Vec<Match>, favorites: &[TeamRef]) -> Vec<Match> {
    if favorites.is_empty() {
        return matches;
    }
    matches
        .into_iter()
        .filter(|game| favorites.iter().any(|team| game.involves(team)))
        .collect()
}
