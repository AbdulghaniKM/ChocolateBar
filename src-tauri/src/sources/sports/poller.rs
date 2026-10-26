use std::collections::HashSet;
use std::time::Duration;

use tauri::async_runtime::block_on;

use crate::core::settings::{SportsSettings, TeamRef};
use crate::core::source::SourceContext;

use super::espn::EspnClient;
use super::model::{keep_favorites, Match};
use super::presenter::{live_activity, live_key, scorebug_activity};
use super::tracker::ScoreTracker;

const LIVE_INTERVAL: Duration = Duration::from_secs(30);
const IDLE_INTERVAL: Duration = Duration::from_secs(10 * 60);
const FAILURE_BASE_INTERVAL: Duration = Duration::from_secs(60);
const MAX_FAILURE_EXPONENT: u32 = 4;

pub struct Poller {
    ctx: SourceContext,
    client: EspnClient,
    tracker: ScoreTracker,
    published: HashSet<String>,
    applied_settings: SportsSettings,
    consecutive_failures: u32,
}

impl Poller {
    pub fn new(ctx: SourceContext, client: EspnClient) -> Self {
        let applied_settings = ctx.current_settings().sports;
        Self {
            ctx,
            client,
            tracker: ScoreTracker::default(),
            published: HashSet::new(),
            applied_settings,
            consecutive_failures: 0,
        }
    }

    pub fn settings_changed(&self) -> bool {
        self.ctx.current_settings().sports != self.applied_settings
    }

    pub fn poll(&mut self) -> Duration {
        let settings = self.ctx.current_settings().sports;
        let fetched = block_on(fetch_scoreboards(&self.client, &settings.leagues));
        self.applied_settings = settings.clone();
        match fetched {
            Some(games) => self.publish_and_schedule(games, &settings.favorite_teams),
            None => self.back_off(),
        }
    }

    fn publish_and_schedule(&mut self, games: Vec<Match>, favorites: &[TeamRef]) -> Duration {
        self.consecutive_failures = 0;
        let visible = keep_favorites(games, favorites);
        for highlight in self.tracker.observe(&visible) {
            self.ctx.upsert(scorebug_activity(&highlight));
        }
        let any_live = self.sync_live_activities(&visible, !favorites.is_empty());
        if any_live {
            LIVE_INTERVAL
        } else {
            IDLE_INTERVAL
        }
    }

    fn sync_live_activities(&mut self, games: &[Match], favorites_only: bool) -> bool {
        let live: Vec<&Match> = games.iter().filter(|game| game.is_live()).collect();
        let keys: HashSet<String> = live.iter().map(|game| live_key(game)).collect();
        for stale_key in self.published.difference(&keys) {
            self.ctx.remove(stale_key);
        }
        for game in &live {
            self.ctx.upsert(live_activity(game, favorites_only));
        }
        self.published = keys;
        !live.is_empty()
    }

    fn back_off(&mut self) -> Duration {
        let exponent = self.consecutive_failures.min(MAX_FAILURE_EXPONENT);
        self.consecutive_failures += 1;
        (FAILURE_BASE_INTERVAL * 2u32.pow(exponent)).min(IDLE_INTERVAL)
    }

    pub fn client(&self) -> &EspnClient {
        &self.client
    }

    pub fn configured_leagues(&self) -> Vec<String> {
        self.ctx.current_settings().sports.leagues
    }
}

async fn fetch_scoreboards(client: &EspnClient, leagues: &[String]) -> Option<Vec<Match>> {
    let mut games = Vec::new();
    let mut any_succeeded = leagues.is_empty();
    for league in leagues {
        match client.scoreboard(league).await {
            Ok(found) => {
                games.extend(found);
                any_succeeded = true;
            }
            Err(error) => log::warn!("sports: scoreboard {league} failed: {error}"),
        }
    }
    any_succeeded.then_some(games)
}
