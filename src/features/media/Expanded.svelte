<script lang="ts">
  import type { Activity } from "../../lib/contracts";
  import Controls from "./Controls.svelte";
  import LyricsView from "./LyricsView.svelte";
  import ProgressBar from "./ProgressBar.svelte";
  import Visualizer from "./Visualizer.svelte";
  import { usePlaybackPosition } from "./playbackClock.svelte";
  import type { MediaExpanded } from "./types";

  let { activity }: { activity: Activity } = $props();

  const media = $derived(activity.expanded as MediaExpanded);
  const playback = usePlaybackPosition(() => media);
</script>

<div class="relative flex h-full flex-col gap-2.5 overflow-hidden px-5 pt-5 pb-3">
  {#if media.art}
    <img class="pointer-events-none absolute inset-0 size-full scale-125 object-cover opacity-25 blur-2xl" src={media.art} alt="" />
  {/if}
  <div class="relative flex items-center gap-4">
    <div class="size-[5.5rem] shrink-0 overflow-hidden rounded-2xl bg-island-raised shadow-lg shadow-black/50 ring-1 ring-white/10">
      {#if media.art}
        <img class="size-full object-cover" src={media.art} alt="" draggable="false" />
      {:else}
        <span class="flex size-full items-center justify-center text-3xl">{activity.compact.icon}</span>
      {/if}
    </div>
    <div class="flex min-w-0 flex-1 flex-col gap-0.5">
      <p class="truncate text-base font-semibold text-island-text">{media.title}</p>
      <p class="truncate text-sm text-island-muted">{media.artist}</p>
      {#if media.album}
        <p class="truncate text-xs text-island-muted/70">{media.album}</p>
      {/if}
    </div>
    <div class="h-[1.5rem] shrink-0 self-start">
      <Visualizer active={media.playing} />
    </div>
  </div>
  <div class="relative">
    <ProgressBar activityId={activity.id} positionMs={playback.positionMs} durationMs={media.durationMs} />
  </div>
  <div class="relative">
    <Controls activityId={activity.id} {media} />
  </div>
  {#if media.lyrics.length > 0}
    <div class="relative">
      <LyricsView lines={media.lyrics} positionMs={playback.positionMs} />
    </div>
  {/if}
</div>
