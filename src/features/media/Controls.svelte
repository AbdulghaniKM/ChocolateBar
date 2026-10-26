<script lang="ts">
  import { activityAction } from "../../lib/ipc";
  import type { MediaExpanded } from "./types";

  let { activityId, media }: { activityId: string; media: MediaExpanded } = $props();

  const send = (action: string) => void activityAction(activityId, action);
  const sideButton =
    "flex size-[2.25rem] items-center justify-center rounded-full text-island-text transition enabled:hover:bg-white/10 enabled:active:scale-90 disabled:opacity-30";
</script>

<div class="flex items-center justify-center gap-3">
  <button class={sideButton} type="button" aria-label="Previous" disabled={!media.canGoPrevious} onclick={() => send("prev")}>
    <svg class="size-[1.125rem]" viewBox="0 0 24 24" fill="currentColor"><path d="M6 5h2v14H6zM20 5v14L9 12z" /></svg>
  </button>
  <button
    class="flex size-[2.75rem] items-center justify-center rounded-full bg-island-text text-island transition hover:scale-105 active:scale-90"
    type="button"
    aria-label={media.playing ? "Pause" : "Play"}
    onclick={() => send("toggle")}
  >
    {#if media.playing}
      <svg class="size-[1.25rem]" viewBox="0 0 24 24" fill="currentColor"><path d="M6 5h4v14H6zM14 5h4v14h-4z" /></svg>
    {:else}
      <svg class="size-[1.25rem]" viewBox="0 0 24 24" fill="currentColor"><path d="M7 4v16l13-8z" /></svg>
    {/if}
  </button>
  <button class={sideButton} type="button" aria-label="Next" disabled={!media.canGoNext} onclick={() => send("next")}>
    <svg class="size-[1.125rem]" viewBox="0 0 24 24" fill="currentColor"><path d="M16 5h2v14h-2zM4 5v14l11-7z" /></svg>
  </button>
</div>
