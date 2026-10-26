use serde::Deserialize;
use serde_json::{json, Value};
use tauri::async_runtime::block_on;

use crate::core::activity::SourceCommand;

use super::espn::EspnClient;
use super::model::TeamEntry;

const TEAMS_COMMAND: &str = "teams";

#[derive(Deserialize, Default)]
#[serde(default)]
struct TeamsRequest {
    leagues: Option<Vec<String>>,
}

pub fn answer(command: SourceCommand, client: &EspnClient, configured_leagues: Vec<String>) {
    if command.command != TEAMS_COMMAND {
        return command.respond_err("command not supported");
    }
    let request: TeamsRequest = serde_json::from_value(command.payload.clone()).unwrap_or_default();
    let leagues = request.leagues.unwrap_or(configured_leagues);
    match block_on(fetch_teams(client, &leagues)) {
        Some(teams) => command.respond_ok(teams_json(&teams)),
        None => command.respond_err("could not load teams"),
    }
}

fn teams_json(teams: &[TeamEntry]) -> Value {
    json!(teams)
}

async fn fetch_teams(client: &EspnClient, leagues: &[String]) -> Option<Vec<TeamEntry>> {
    let mut teams = Vec::new();
    let mut any_succeeded = leagues.is_empty();
    for league in leagues {
        match client.teams(league).await {
            Ok(found) => {
                teams.extend(found);
                any_succeeded = true;
            }
            Err(error) => log::warn!("sports: teams {league} failed: {error}"),
        }
    }
    teams.sort_by(|a, b| a.league.cmp(&b.league).then_with(|| a.name.cmp(&b.name)));
    any_succeeded.then_some(teams)
}
