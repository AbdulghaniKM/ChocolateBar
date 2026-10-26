use crate::core::activity::{Activity, ActivityKind, CompactView, Priority};

use super::model::Match;
use super::tracker::{Highlight, HighlightReason};

const SOURCE_NAME: &str = "sports";
const SCOREBUG_TTL_MS: u64 = 6_000;
const LIVE_ACCENT: &str = "#ef4444";
const SCORE_ACCENT: &str = "#22c55e";

pub fn live_key(game: &Match) -> String {
    format!("live:{}:{}", game.league, game.id)
}

pub fn live_activity(game: &Match, favorites_only: bool) -> Activity {
    let priority = if favorites_only { Priority::Normal } else { Priority::Ambient };
    let compact = compact_view(game).accent(LIVE_ACCENT);
    expanded_activity(&live_key(game), game, compact).priority(priority)
}

pub fn scorebug_activity(highlight: &Highlight) -> Activity {
    let game = &highlight.game;
    let badge = match highlight.reason {
        HighlightReason::ScoreChanged => "SCORE",
        HighlightReason::FullTime => "FT",
    };
    let compact = compact_view(game).accent(SCORE_ACCENT).badge(badge);
    let key = format!("bug:{}:{}", game.league, game.id);
    expanded_activity(&key, game, compact).priority(Priority::High).ttl_ms(SCOREBUG_TTL_MS)
}

fn expanded_activity(key: &str, game: &Match, compact: CompactView) -> Activity {
    let expanded = serde_json::to_value(game).unwrap_or_default();
    Activity::new(SOURCE_NAME, key, ActivityKind::Sports, compact).expanded(expanded)
}

fn compact_view(game: &Match) -> CompactView {
    CompactView::new(sport_icon(&game.sport), scoreline_title(game)).subtitle(game.status.clone())
}

fn scoreline_title(game: &Match) -> String {
    format!(
        "{} {}\u{2013}{} {}",
        game.home.abbreviation, game.home.score, game.away.score, game.away.abbreviation
    )
}

fn sport_icon(sport: &str) -> &'static str {
    if sport == "basketball" {
        "\u{1F3C0}"
    } else {
        "\u{26BD}"
    }
}
