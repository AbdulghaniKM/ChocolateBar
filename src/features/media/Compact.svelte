<script lang="ts">
  import EighthNotes from "@lucide/svelte/icons/eighth-notes";
  import type { Activity } from "../../lib/contracts";
  import { settingsState } from "../../lib/settings.svelte";
  import Visualizer from "./Visualizer.svelte";
  import type { MediaExpanded } from "./types";

  let { activity }: { activity: Activity } = $props();

  const media = $derived(activity.expanded as MediaExpanded);
</script>

<div class="flex h-full w-full min-w-0 items-center gap-2.5 pr-3.5 pl-1.5">
  <div class="size-7 shrink-0 overflow-hidden rounded-[0.625rem] bg-island-raised ring-1 ring-white/10">
    {#if media.art}
      <img class="size-full object-cover" src={media.art} alt="" draggable="false" />
    {:else}
      <span class="grid size-full place-items-center text-island-subtle"><EighthNotes class="size-4" /></span>
    {/if}
  </div>
  <div class="flex min-w-0 flex-1 flex-col leading-tight">
    <span class="truncate text-[0.8125rem] font-medium text-island-text">{media.title}</span>
    <span class="truncate text-[0.6875rem] text-island-muted">{media.artist || media.source?.app}</span>
  </div>
  {#if settingsState.value.media.visualizer}
    <div class="h-4 shrink-0">
      <Visualizer active={media.playing} />
    </div>
  {/if}
</div>
