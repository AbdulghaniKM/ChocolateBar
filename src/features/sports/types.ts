export interface MatchSide {
  id: string;
  name: string;
  abbreviation: string;
  logo: string | null;
  score: number;
}

export interface MatchView {
  id: string;
  sport: "soccer" | "basketball";
  league: string;
  state: "upcoming" | "live" | "finished";
  status: string;
  home: MatchSide;
  away: MatchSide;
}

export interface TeamEntry {
  sport: string;
  league: string;
  id: string;
  name: string;
  logo: string | null;
}

export interface LeagueOption {
  id: string;
  name: string;
}

export const LEAGUE_OPTIONS: LeagueOption[] = [
  { id: "eng.1", name: "Premier League" },
  { id: "esp.1", name: "La Liga" },
  { id: "ger.1", name: "Bundesliga" },
  { id: "ita.1", name: "Serie A" },
  { id: "fra.1", name: "Ligue 1" },
  { id: "uefa.champions", name: "Champions League" },
  { id: "uefa.europa", name: "Europa League" },
  { id: "usa.1", name: "MLS" },
  { id: "nba", name: "NBA" },
  { id: "wnba", name: "WNBA" },
];

export const leagueName = (id: string) =>
  LEAGUE_OPTIONS.find((league) => league.id === id)?.name ?? id;
