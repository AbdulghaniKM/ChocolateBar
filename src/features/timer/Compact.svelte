<script lang="ts">
  import Glyph from "../../shell/Glyph.svelte";
  import type { Activity } from "../../lib/contracts";
  import Ring from "./Ring.svelte";
  import { useNow } from "./ticker.svelte";
  import { liveElapsedMs, ringFraction, clockTextFor, type TimerView } from "./timing";

  let { activity }: { activity: Activity } = $props();

  const now = useNow(250);
  const view = $derived(activity.expanded as TimerView);
  const isAlert = $derived(view.mode === "alert");
  const elapsedMs = $derived(liveElapsedMs(view, activity.updatedAtMs, now.value));
</script>

{#if isAlert}
  <div class="flex h-full min-w-0 items-center gap-2 px-3 text-sm">
    <span class="animate-pulse text-accent"><Glyph glyph={activity.compact.icon} class="size-[1.125rem]" /></span>
    <span class="truncate font-medium">{activity.compact.title}</span>
    {#if activity.compact.subtitle}
      <span class="truncate text-xs text-island-muted">{activity.compact.subtitle}</span>
    {/if}
  </div>
{:else}
  <div class="flex h-full w-full min-w-0 items-center gap-3 px-3 text-sm">
    <div class="size-7 shrink-0">
      <Ring fraction={ringFraction(view, elapsedMs)} accent={view.accent} strokeWidth={10}>
        <Glyph glyph={activity.compact.icon} class="size-3" />
      </Ring>
    </div>
    <span
      class="font-medium tabular-nums transition-opacity duration-300"
      class:opacity-60={!view.running}
    >
      {clockTextFor(view, elapsedMs)}
    </span>
    <span class="min-w-0 truncate text-xs text-island-muted">{activity.compact.subtitle}</span>
  </div>
{/if}
