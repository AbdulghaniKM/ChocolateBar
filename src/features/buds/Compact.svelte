<script lang="ts">
  import Glyph from "../../shell/Glyph.svelte";
  import type { Activity } from "../../lib/contracts";
  import { isStatusView, LOW_BATTERY_PERCENT, type BudsExpanded } from "./types";

  let { activity }: { activity: Activity } = $props();

  const view = $derived(activity.expanded as BudsExpanded);
  const status = $derived(isStatusView(view) ? view : null);
  const levels = $derived(
    status?.link === "connected"
      ? [
          { label: "L", percent: status.left.percent },
          { label: "R", percent: status.right.percent },
          { label: "C", percent: status.case.percent },
        ].filter((level) => level.percent !== null)
      : [],
  );
</script>

<div class="flex h-full w-full min-w-0 items-center gap-2.5 px-4">
  <Glyph glyph={activity.compact.icon} class="size-[1.125rem] text-island-text" />
  {#if levels.length > 0}
    <span class="min-w-0 truncate text-[0.8125rem] font-medium text-island-text">{activity.compact.title}</span>
    <div class="ml-auto flex shrink-0 items-center gap-1.5">
      {#each levels as level (level.label)}
        <span
          class="rounded-pill bg-island-raised px-1.5 py-0.5 text-[0.625rem] font-semibold tabular-nums
            {(level.percent ?? 100) <= LOW_BATTERY_PERCENT ? 'text-danger' : 'text-island-text'}"
        >
          {level.label} {level.percent}
        </span>
      {/each}
    </div>
  {:else}
    <div class="flex min-w-0 flex-1 flex-col leading-tight">
      <span class="truncate text-[0.8125rem] font-medium text-island-text">{activity.compact.title}</span>
      <span class="truncate text-[0.6875rem] text-island-muted">{activity.compact.subtitle}</span>
    </div>
  {/if}
  {#if activity.compact.badge && levels.length > 0}
    <span class="rounded-pill bg-accent/15 px-2 py-0.5 text-[0.625rem] font-semibold text-accent">
      {activity.compact.badge}
    </span>
  {/if}
</div>
