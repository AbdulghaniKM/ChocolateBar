<script lang="ts">
  import EighthNotes from "@lucide/svelte/icons/eighth-notes";
  import type { Activity } from "../../lib/contracts";
  import Controls from "./Controls.svelte";
  import LyricsView from "./LyricsView.svelte";
  import ProgressBar from "./ProgressBar.svelte";
  import SourceSwitcher from "./SourceSwitcher.svelte";
  import { usePlaybackPosition } from "./playbackClock.svelte";
  import type { MediaExpanded } from "./types";

  let { activity }: { activity: Activity } = $props();

  const media = $derived(activity.expanded as MediaExpanded);
  const playback = usePlaybackPosition(() => media);
  const hasSources = $derived((media.sources?.length ?? 0) > 1);
</script>

<div class="relative flex h-full flex-col gap-3 px-5 pb-4">
  {#if media.art}
    <img
      class="pointer-events-none absolute inset-x-0 -top-12 bottom-0 size-full scale-125 object-cover opacity-20 blur-3xl saturate-150"
      src={media.art}
      alt=""
    />
  {/if}
  <div class="relative flex items-center gap-4">
    <div class="size-[4.5rem] shrink-0 overflow-hidden rounded-xl bg-island-raised shadow-lg shadow-black/50 ring-1 ring-white/10">
      {#if media.art}
        <img class="size-full object-cover" src={media.art} alt="" draggable="false" />
      {:else}
        <span class="grid size-full place-items-center text-island-subtle"><EighthNotes class="size-7" /></span>
      {/if}
    </div>
    <div class="flex min-w-0 flex-1 flex-col gap-0.5">
      <p class="line-clamp-2 text-[1rem] leading-snug font-semibold text-island-text" title={media.title}>{media.title}</p>
      <p class="truncate text-[0.8125rem] text-island-muted">{media.artist || media.source?.app}</p>
      {#if media.album}
        <p class="truncate text-[0.75rem] text-island-subtle">{media.album}</p>
      {/if}
    </div>
  </div>
  <div class="relative">
    <ProgressBar activityId={activity.id} positionMs={playback.positionMs} durationMs={media.durationMs} />
  </div>
  <div class="relative">
    <Controls activityId={activity.id} {media} />
  </div>
  <div class="relative min-h-0 flex-1">
    {#if media.lyrics.length > 0}
      <LyricsView lines={media.lyrics} positionMs={playback.positionMs} />
    {/if}
  </div>
  {#if hasSources}
    <div class="relative">
      <SourceSwitcher activityId={activity.id} {media} />
    </div>
  {/if}
</div>
