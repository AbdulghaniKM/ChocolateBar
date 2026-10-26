<script lang="ts">
  import type { Activity } from "../lib/contracts";
  import { registry } from "../features/registry";
  import Glyph from "./Glyph.svelte";

  let { activity }: { activity: Activity } = $props();

  const Compact = $derived(registry[activity.kind].Compact);
  const progressPercent = $derived(Math.round((activity.compact.progress ?? 0) * 100));
</script>

<div class="relative flex h-full w-full items-center">
  {#if Compact}
    <Compact {activity} />
  {:else}
    <div class="flex min-w-0 items-center gap-2.5 px-3.5 text-[0.8125rem]">
      <Glyph glyph={activity.compact.icon} class="size-4 text-accent" />
      <span class="truncate font-medium">{activity.compact.title}</span>
      {#if activity.compact.subtitle}
        <span class="truncate text-island-muted">{activity.compact.subtitle}</span>
      {/if}
      {#if activity.compact.badge}
        <span class="ml-auto rounded-pill bg-island-raised px-2 text-xs">{activity.compact.badge}</span>
      {/if}
    </div>
  {/if}
  {#if activity.compact.progress !== null}
    <div class="absolute inset-x-5 bottom-0 h-[0.1875rem] overflow-hidden rounded-pill bg-island-raised">
      <div
        class="h-full rounded-pill bg-accent transition-[width] duration-300"
        style:width="{progressPercent}%"
        style:background-color={activity.compact.accent}
      ></div>
    </div>
  {/if}
</div>
