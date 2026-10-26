<script lang="ts">
  import { onMount } from "svelte";
  import { sourceCommand } from "../../lib/ipc";
  import { formatDuration, type PlaytimeEntry } from "./types";

  let entries = $state<PlaytimeEntry[]>([]);
  let failed = $state(false);

  async function loadPlaytime() {
    failed = false;
    try {
      entries = await sourceCommand<PlaytimeEntry[]>("game", "playtime");
    } catch (error) {
      console.warn("playtime failed", error);
      failed = true;
    }
  }

  onMount(() => void loadPlaytime());
</script>

<section class="flex flex-col gap-4">
  <h2 class="text-lg font-medium">Games</h2>
  <p class="text-sm text-island-muted">
    Steam and Epic Games downloads show up automatically. Playtime is tracked while a game from your
    libraries is running.
  </p>

  <div class="flex items-center justify-between">
    <h3 class="text-sm font-medium">Playtime</h3>
    <button
      class="rounded-lg bg-island-raised px-3 py-1 text-xs transition-colors hover:bg-island-border"
      type="button"
      onclick={loadPlaytime}
    >
      Refresh
    </button>
  </div>

  {#if failed}
    <p class="text-sm text-island-muted">Could not load playtime. Is the game source enabled?</p>
  {:else if entries.length === 0}
    <p class="text-sm text-island-muted">No playtime recorded yet.</p>
  {:else}
    <ul class="flex flex-col gap-1">
      {#each entries as entry (entry.name)}
        <li class="flex items-center justify-between rounded-lg bg-island-raised px-3 py-2 text-sm">
          <span class="truncate">{entry.name}</span>
          <span class="shrink-0 tabular-nums text-island-muted">{formatDuration(entry.seconds)}</span>
        </li>
      {/each}
    </ul>
  {/if}
</section>
