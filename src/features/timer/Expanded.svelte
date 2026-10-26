<script lang="ts">
  import Glyph from "../../shell/Glyph.svelte";
  import type { Activity } from "../../lib/contracts";
  import Ring from "./Ring.svelte";
  import { useNow } from "./ticker.svelte";
  import {
    clockTextFor,
    formatLap,
    liveElapsedMs,
    ringFraction,
    type TimerView,
  } from "./timing";

  let { activity }: { activity: Activity } = $props();

  const now = useNow();
  const view = $derived(activity.expanded as TimerView);
  const elapsedMs = $derived(liveElapsedMs(view, activity.updatedAtMs, now.value));
  const isStopwatch = $derived(view.mode === "stopwatch");
  const sessionSlots = $derived(
    view.session ? Array.from({ length: view.session.of }, (_, index) => index) : [],
  );
</script>

{#if view.mode === "alert"}
  <div class="flex h-full flex-col items-center justify-center gap-3 px-6 text-center">
    <span class="animate-pulse text-accent"><Glyph glyph={activity.compact.icon} class="size-12" strokeWidth={1.75} /></span>
    <p class="text-xl font-semibold">{activity.compact.title}</p>
    <p class="text-sm text-island-muted">{view.label}</p>
  </div>
{:else}
  <div class="flex h-full items-center gap-6 px-6">
    <div class="size-[10rem] shrink-0">
      <Ring fraction={ringFraction(view, elapsedMs)} accent={view.accent}>
        <span
          class="text-3xl font-light tabular-nums transition-opacity duration-300"
          class:opacity-60={!view.running}
        >
          {clockTextFor(view, elapsedMs)}
        </span>
        <span class="text-xs tracking-widest text-island-muted uppercase">{view.label}</span>
      </Ring>
    </div>

    <div class="flex min-h-0 min-w-0 flex-1 flex-col gap-3 self-stretch py-6">
      {#if view.session}
        <div class="flex items-center gap-2" aria-label="Pomodoro sessions">
          {#each sessionSlots as slot (slot)}
            <span
              class="size-2.5 rounded-full transition-colors duration-300"
              style:background-color={slot < view.session.completed ? view.accent : "var(--color-island-raised)"}
            ></span>
          {/each}
          <span class="text-xs text-island-muted">{view.session.completed}/{view.session.of}</span>
        </div>
      {/if}

      {#if isStopwatch}
        <ul class="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto pr-1 text-sm tabular-nums">
          {#each view.laps as lap (lap.index)}
            <li class="flex items-center justify-between rounded-lg bg-island-raised px-3 py-1">
              <span class="text-island-muted">Lap {lap.index}</span>
              <span>{formatLap(lap.splitMs)}</span>
            </li>
          {:else}
            <li class="text-sm text-island-muted">Press Lap to record a split.</li>
          {/each}
        </ul>
      {:else}
        <p class="text-sm text-island-muted">
          {view.running ? "Stay with it." : "Paused. Resume when you are ready."}
        </p>
      {/if}
    </div>
  </div>
{/if}
