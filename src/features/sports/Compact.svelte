<script lang="ts">
  import type { Activity } from "../../lib/contracts";
  import type { MatchView } from "./types";
  import TeamLogo from "./TeamLogo.svelte";

  let { activity }: { activity: Activity } = $props();

  const match = $derived(activity.expanded as MatchView);
  const isScorebug = $derived(activity.compact.badge !== null);
</script>

<div class="flex h-full w-full min-w-0 items-center justify-between gap-2 px-3 text-sm">
  <div class="flex min-w-0 items-center gap-1.5">
    <TeamLogo logo={match.home.logo} name={match.home.name} class="size-5" />
    <span class="truncate text-xs font-semibold tracking-wide">{match.home.abbreviation}</span>
  </div>

  <div
    class="flex shrink-0 items-center gap-1.5 rounded-full bg-island-raised px-2.5 py-0.5 font-semibold tabular-nums transition-colors duration-300"
    class:scorebug={isScorebug}
  >
    <span>{match.home.score}</span>
    <span class="text-island-muted">&ndash;</span>
    <span>{match.away.score}</span>
  </div>

  <div class="flex min-w-0 items-center gap-1.5">
    <span class="truncate text-xs font-semibold tracking-wide">{match.away.abbreviation}</span>
    <TeamLogo logo={match.away.logo} name={match.away.name} class="size-5" />
  </div>
</div>

<style>
  .scorebug {
    animation: scorebug-pulse 1.2s ease-out 2;
    background-color: color-mix(in srgb, var(--color-success) 35%, var(--color-island-raised));
  }

  @keyframes scorebug-pulse {
    0% {
      box-shadow: 0 0 0 0 color-mix(in srgb, var(--color-success) 70%, transparent);
    }
    100% {
      box-shadow: 0 0 0 0.6rem transparent;
    }
  }
</style>
