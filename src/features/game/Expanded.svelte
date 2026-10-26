<script lang="ts">
  import type { Activity } from "../../lib/contracts";
  import { useNow } from "../timer/ticker.svelte";
  import {
    PLAYING_ACTIVITY_ID,
    formatBytes,
    formatDuration,
    type DownloadView,
    type PlayingView,
  } from "./types";

  let { activity }: { activity: Activity } = $props();

  const now = useNow(1000);
  const isPlaying = $derived(activity.id === PLAYING_ACTIVITY_ID);
  const download = $derived(activity.expanded as DownloadView);
  const playing = $derived(activity.expanded as PlayingView);
  const sessionSeconds = $derived(Math.max(0, Math.floor((now.value - playing.startedAtMs) / 1000)));
  const percent = $derived(Math.round((download.fraction ?? 0) * 100));
</script>

{#if isPlaying}
  <div class="flex h-full flex-col items-center justify-center gap-3 px-6 text-center">
    <span class="flex items-center gap-2 text-xs tracking-widest text-success uppercase">
      <span class="size-2 animate-pulse rounded-full bg-success"></span>
      Now playing
    </span>
    <p class="max-w-full truncate text-2xl font-semibold">{playing.name}</p>
    <p class="text-5xl font-light tabular-nums">{formatDuration(sessionSeconds)}</p>
    <p class="text-sm text-island-muted">
      {formatDuration(playing.totalSeconds - playing.sessionSeconds + sessionSeconds)} total
    </p>
  </div>
{:else}
  <div class="flex h-full flex-col justify-center gap-4 px-6">
    <div class="flex flex-col gap-1">
      <span class="text-xs tracking-widest text-island-muted uppercase">{download.launcher}</span>
      <p class="truncate text-xl font-semibold">{download.name}</p>
    </div>

    <div class="flex flex-col gap-2">
      <div class="h-2 overflow-hidden rounded-full bg-island-raised">
        {#if download.fraction === null}
          <div class="indeterminate h-full w-1/3 rounded-full bg-accent"></div>
        {:else}
          <div
            class="h-full rounded-full bg-accent transition-[width] duration-500 ease-island"
            class:opacity-50={download.paused}
            style:width="{percent}%"
          ></div>
        {/if}
      </div>
      <div class="flex items-center justify-between text-sm tabular-nums">
        <span class="text-island-muted">
          {#if download.fraction === null}
            Installing
          {:else if download.paused}
            Paused &middot; {percent}%
          {:else}
            {percent}%
          {/if}
        </span>
        {#if download.totalBytes}
          <span class="text-island-muted">
            {download.downloadedBytes === null
              ? formatBytes(download.totalBytes)
              : `${formatBytes(download.downloadedBytes)} / ${formatBytes(download.totalBytes)}`}
          </span>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .indeterminate {
    animation: slide 1.4s var(--ease-island) infinite;
  }

  @keyframes slide {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(300%);
    }
  }
</style>
