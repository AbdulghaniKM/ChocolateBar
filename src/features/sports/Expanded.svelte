<script lang="ts">
  import type { Activity } from "../../lib/contracts";
  import { leagueName, type MatchView } from "./types";
  import TeamLogo from "./TeamLogo.svelte";

  let { activity }: { activity: Activity } = $props();

  const match = $derived(activity.expanded as MatchView);
  const isLive = $derived(match.state === "live");
</script>

<div class="flex h-full flex-col items-center justify-center gap-4 px-6">
  <div class="flex items-center gap-2 text-xs text-island-muted">
    {#if isLive}
      <span class="size-2 animate-pulse rounded-full bg-danger"></span>
    {/if}
    <span class="uppercase tracking-widest">{leagueName(match.league)}</span>
  </div>

  <div class="grid w-full grid-cols-[1fr_auto_1fr] items-center gap-4">
    <div class="flex min-w-0 flex-col items-center gap-2">
      <TeamLogo logo={match.home.logo} name={match.home.name} class="size-16" />
      <span class="w-full truncate text-center text-sm font-medium">{match.home.name}</span>
    </div>

    <div class="flex flex-col items-center gap-1">
      <div class="flex items-center gap-3 text-4xl font-semibold tabular-nums">
        <span>{match.home.score}</span>
        <span class="text-island-muted">:</span>
        <span>{match.away.score}</span>
      </div>
      <span class="rounded-full bg-island-raised px-3 py-0.5 text-xs text-island-muted">
        {match.status}
      </span>
    </div>

    <div class="flex min-w-0 flex-col items-center gap-2">
      <TeamLogo logo={match.away.logo} name={match.away.name} class="size-16" />
      <span class="w-full truncate text-center text-sm font-medium">{match.away.name}</span>
    </div>
  </div>
</div>
