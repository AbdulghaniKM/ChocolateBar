use std::time::Duration;

use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;

use super::model::{Match, MatchState, Side, TeamEntry};

const API_ROOT: &str = "https://site.api.espn.com/apis/site/v2/sports";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const USER_AGENT: &str = concat!("ChocolateBar/", env!("CARGO_PKG_VERSION"));

pub fn sport_of_league(league: &str) -> &'static str {
    match league {
        "nba" | "wnba" | "mens-college-basketball" | "womens-college-basketball" => "basketball",
        _ => "soccer",
    }
}

fn league_url(league: &str, resource: &str) -> String {
    format!("{API_ROOT}/{}/{league}/{resource}", sport_of_league(league))
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct ScoreboardDto {
    events: Option<Vec<EventDto>>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct EventDto {
    id: Option<String>,
    status: Option<StatusDto>,
    competitions: Option<Vec<CompetitionDto>>,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct StatusDto {
    display_clock: Option<String>,
    #[serde(rename = "type")]
    kind: Option<StatusKindDto>,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct StatusKindDto {
    state: Option<String>,
    short_detail: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct CompetitionDto {
    competitors: Option<Vec<CompetitorDto>>,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct CompetitorDto {
    home_away: Option<String>,
    score: Option<Value>,
    team: Option<TeamDto>,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct TeamDto {
    id: Option<String>,
    display_name: Option<String>,
    short_display_name: Option<String>,
    abbreviation: Option<String>,
    logo: Option<String>,
    logos: Option<Vec<LogoDto>>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct LogoDto {
    href: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct TeamsDto {
    sports: Option<Vec<SportDto>>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct SportDto {
    leagues: Option<Vec<LeagueDto>>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct LeagueDto {
    teams: Option<Vec<TeamWrapperDto>>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct TeamWrapperDto {
    team: Option<TeamDto>,
}

impl TeamDto {
    fn logo_url(&self) -> Option<String> {
        self.logo
            .clone()
            .or_else(|| self.logos.as_ref()?.iter().find_map(|logo| logo.href.clone()))
    }

    fn full_name(&self) -> Option<String> {
        self.display_name.clone().or_else(|| self.short_display_name.clone())
    }

    fn into_side(self, score: u32) -> Option<Side> {
        let name = self.full_name()?;
        Some(Side {
            id: self.id.clone()?,
            abbreviation: self.abbreviation.clone().unwrap_or_else(|| name.clone()),
            logo: self.logo_url(),
            name,
            score,
        })
    }

    fn into_entry(self, league: &str) -> Option<TeamEntry> {
        Some(TeamEntry {
            sport: sport_of_league(league).into(),
            league: league.into(),
            logo: self.logo_url(),
            id: self.id.clone()?,
            name: self.full_name()?,
        })
    }
}

fn score_of(value: Option<&Value>) -> u32 {
    match value {
        Some(Value::String(text)) => text.trim().parse().unwrap_or_default(),
        Some(Value::Number(number)) => number.as_u64().unwrap_or_default() as u32,
        _ => 0,
    }
}

fn state_of(raw: Option<&str>) -> MatchState {
    match raw {
        Some("in") => MatchState::Live,
        Some("post") => MatchState::Finished,
        _ => MatchState::Upcoming,
    }
}

fn side_of(competitor: CompetitorDto) -> Option<Side> {
    let score = score_of(competitor.score.as_ref());
    competitor.team?.into_side(score)
}

fn take_side(competitors: &mut Vec<CompetitorDto>, home_away: &str) -> Option<Side> {
    let position = competitors
        .iter()
        .position(|competitor| competitor.home_away.as_deref() == Some(home_away))?;
    side_of(competitors.swap_remove(position))
}

impl EventDto {
    fn into_match(self, league: &str) -> Option<Match> {
        let mut competitors = self.competitions?.into_iter().next()?.competitors?;
        let status = self.status.unwrap_or_default();
        let kind = status.kind.unwrap_or_default();
        Some(Match {
            id: self.id?,
            sport: sport_of_league(league).into(),
            league: league.into(),
            state: state_of(kind.state.as_deref()),
            status: kind.short_detail.or(status.display_clock).unwrap_or_default(),
            home: take_side(&mut competitors, "home")?,
            away: take_side(&mut competitors, "away")?,
        })
    }
}

pub struct EspnClient {
    http: Client,
}

impl EspnClient {
    pub fn new() -> reqwest::Result<Self> {
        let http = Client::builder().user_agent(USER_AGENT).timeout(REQUEST_TIMEOUT).build()?;
        Ok(Self { http })
    }

    pub async fn scoreboard(&self, league: &str) -> reqwest::Result<Vec<Match>> {
        let dto: ScoreboardDto = self.get_json(&league_url(league, "scoreboard")).await?;
        let events = dto.events.unwrap_or_default();
        Ok(events.into_iter().filter_map(|event| event.into_match(league)).collect())
    }

    pub async fn teams(&self, league: &str) -> reqwest::Result<Vec<TeamEntry>> {
        let url = format!("{}?limit=100", league_url(league, "teams"));
        let dto: TeamsDto = self.get_json(&url).await?;
        let teams = dto
            .sports
            .into_iter()
            .flatten()
            .flat_map(|sport| sport.leagues.into_iter().flatten())
            .flat_map(|league_dto| league_dto.teams.into_iter().flatten())
            .filter_map(|wrapper| wrapper.team?.into_entry(league));
        Ok(teams.collect())
    }

    async fn get_json<T: DeserializeOwned>(&self, url: &str) -> reqwest::Result<T> {
        self.http.get(url).send().await?.error_for_status()?.json().await
    }
}
