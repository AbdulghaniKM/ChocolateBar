<script lang="ts">
  import { onMount } from "svelte";
  import type { TeamRef } from "../../lib/contracts";
  import { sourceCommand } from "../../lib/ipc";
  import { settingsState } from "../../lib/settings.svelte";
  import { LEAGUE_OPTIONS, leagueName, type TeamEntry } from "./types";
  import TeamLogo from "./TeamLogo.svelte";

  let teams = $state<TeamEntry[]>([]);
  let loading = $state(false);
  let failed = $state(false);
  let search = $state("");

  const sports = $derived(settingsState.value.sports);
  const favoriteKeys = $derived(new Set(sports.favoriteTeams.map(teamKey)));
  const visibleTeams = $derived(
    teams.filter((team) => team.name.toLowerCase().includes(search.trim().toLowerCase())),
  );

  const RELOAD_DELAY_MS = 400;
  let reloadTimer: ReturnType<typeof setTimeout> | undefined;

  function teamKey(team: { league: string; id: string }) {
    return `${team.league}:${team.id}`;
  }

  function toTeamRef(team: TeamEntry): TeamRef {
    return { sport: team.sport, league: team.league, id: team.id, name: team.name };
  }

  async function loadTeams() {
    loading = true;
    failed = false;
    try {
      teams = await sourceCommand<TeamEntry[]>("sports", "teams", { leagues: sports.leagues });
    } catch (error) {
      console.warn("sports teams failed", error);
      failed = true;
    } finally {
      loading = false;
    }
  }

  function toggleLeague(leagueId: string, enabled: boolean) {
    const leagues = enabled
      ? [...sports.leagues, leagueId]
      : sports.leagues.filter((league) => league !== leagueId);
    void settingsState.update({ sports: { leagues } });
    clearTimeout(reloadTimer);
    reloadTimer = setTimeout(loadTeams, RELOAD_DELAY_MS);
  }

  function toggleFavorite(team: TeamEntry) {
    const favoriteTeams = favoriteKeys.has(teamKey(team))
      ? sports.favoriteTeams.filter((favorite) => teamKey(favorite) !== teamKey(team))
      : [...sports.favoriteTeams, toTeamRef(team)];
    void settingsState.update({ sports: { favoriteTeams } });
  }

  onMount(() => {
    void loadTeams();
    return () => clearTimeout(reloadTimer);
  });
</script>

<section class="flex flex-col gap-5">
  <h2 class="text-lg font-medium">Sports</h2>

  <div class="flex flex-col gap-2">
    <h3 class="text-sm font-medium">Leagues</h3>
    <div class="grid grid-cols-2 gap-2">
      {#each LEAGUE_OPTIONS as league (league.id)}
        <label class="flex items-center gap-3 text-sm">
          <input
            type="checkbox"
            class="accent-accent"
            checked={sports.leagues.includes(league.id)}
            onchange={(event) => toggleLeague(league.id, event.currentTarget.checked)}
          />
          {league.name}
        </label>
      {/each}
    </div>
  </div>

  <div class="flex flex-col gap-2">
    <div class="flex items-center justify-between">
      <h3 class="text-sm font-medium">Favorite teams</h3>
      <span class="text-xs text-island-muted">
        {sports.favoriteTeams.length === 0
          ? "No filter: all live games are shown"
          : `Only ${sports.favoriteTeams.length} selected team(s) are shown`}
      </span>
    </div>

    {#if sports.favoriteTeams.length > 0}
      <div class="flex flex-wrap gap-2">
        {#each sports.favoriteTeams as favorite (teamKey(favorite))}
          <button
            type="button"
            class="rounded-full bg-accent px-3 py-1 text-xs font-medium text-island transition-opacity hover:opacity-80"
            onclick={() => void settingsState.update({
              sports: {
                favoriteTeams: sports.favoriteTeams.filter((team) => teamKey(team) !== teamKey(favorite)),
              },
            })}
          >
            {favorite.name} &times;
          </button>
        {/each}
      </div>
    {/if}

    <input
      class="rounded-lg bg-island-raised px-3 py-2 text-sm outline-none focus:ring-2 focus:ring-accent"
      type="search"
      placeholder="Search teams"
      bind:value={search}
    />

    <div class="flex max-h-[18rem] flex-col gap-1 overflow-y-auto rounded-lg border border-island-border p-1">
      {#if loading}
        <p class="p-3 text-sm text-island-muted">Loading teams...</p>
      {:else if failed}
        <div class="flex items-center justify-between p-3 text-sm text-island-muted">
          <span>Could not load teams.</span>
          <button type="button" class="rounded-lg bg-island-raised px-3 py-1 text-island-text" onclick={loadTeams}>
            Retry
          </button>
        </div>
      {:else if visibleTeams.length === 0}
        <p class="p-3 text-sm text-island-muted">No teams found.</p>
      {:else}
        {#each visibleTeams as team (teamKey(team))}
          <button
            type="button"
            class="flex items-center gap-3 rounded-lg px-3 py-1.5 text-left text-sm transition-colors hover:bg-island-raised"
            class:bg-island-raised={favoriteKeys.has(teamKey(team))}
            onclick={() => toggleFavorite(team)}
          >
            <TeamLogo logo={team.logo} name={team.name} class="size-6" />
            <span class="flex-1 truncate">{team.name}</span>
            <span class="text-xs text-island-muted">{leagueName(team.league)}</span>
            <span class="w-4 text-accent">{favoriteKeys.has(teamKey(team)) ? "★" : ""}</span>
          </button>
        {/each}
      {/if}
    </div>
  </div>
</section>
