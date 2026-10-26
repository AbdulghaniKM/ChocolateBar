<script lang="ts">
  import type { Activity } from "../../lib/contracts";
  import Visualizer from "./Visualizer.svelte";
  import type { MediaExpanded } from "./types";

  let { activity }: { activity: Activity } = $props();

  const media = $derived(activity.expanded as MediaExpanded);
</script>

<div class="flex h-full w-full min-w-0 items-center gap-2.5 px-2.5">
  <div class="size-[1.75rem] shrink-0 overflow-hidden rounded-lg bg-island-raised ring-1 ring-white/10">
    {#if media.art}
      <img class="size-full object-cover" src={media.art} alt="" draggable="false" />
    {:else}
      <span class="flex size-full items-center justify-center text-sm">{activity.compact.icon}</span>
    {/if}
  </div>
  <div class="flex min-w-0 flex-1 flex-col leading-tight">
    <span class="truncate text-[0.8125rem] font-medium text-island-text">{media.title}</span>
    {#if media.artist}
      <span class="truncate text-[0.6875rem] text-island-muted">{media.artist}</span>
    {/if}
  </div>
  <div class="h-[1.125rem] shrink-0">
    <Visualizer active={media.playing} />
  </div>
</div>
