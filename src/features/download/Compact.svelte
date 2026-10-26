<script lang="ts">
  import Glyph from "../../shell/Glyph.svelte";
  import type { Activity } from "../../lib/contracts";
  import IndeterminateBar from "./IndeterminateBar.svelte";
  import type { DownloadView } from "./types";

  let { activity }: { activity: Activity } = $props();

  const download = $derived(activity.expanded as DownloadView);
</script>

<div class="relative flex h-full w-full min-w-0 items-center gap-2.5 px-3.5">
  <span class="{download.done ? 'text-success' : 'arrow text-info'}"><Glyph glyph={activity.compact.icon} class="size-4" /></span>
  <div class="flex min-w-0 flex-1 flex-col leading-tight">
    <span class="truncate text-[0.8125rem] font-medium text-island-text">{download.fileName}</span>
    <span class="truncate text-[0.6875rem] text-island-muted tabular-nums">{activity.compact.subtitle}</span>
  </div>
  {#if !download.done}
    <div class="absolute inset-x-4 bottom-0">
      <IndeterminateBar />
    </div>
  {/if}
</div>

<style>
  .arrow {
    animation: nudge 1.2s ease-in-out infinite;
  }
  @keyframes nudge {
    50% {
      transform: translateY(0.18rem);
    }
  }
</style>
