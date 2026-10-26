<script lang="ts">
  import Pause from "@lucide/svelte/icons/pause";
  import Play from "@lucide/svelte/icons/play";
  import Repeat from "@lucide/svelte/icons/repeat";
  import Repeat1 from "@lucide/svelte/icons/repeat-1";
  import Shuffle from "@lucide/svelte/icons/shuffle";
  import SkipBack from "@lucide/svelte/icons/skip-back";
  import SkipForward from "@lucide/svelte/icons/skip-forward";
  import { activityAction } from "../../lib/ipc";
  import type { MediaExpanded, RepeatMode } from "./types";

  let { activityId, media }: { activityId: string; media: MediaExpanded } = $props();

  const NEXT_REPEAT: Record<RepeatMode, RepeatMode> = { off: "list", list: "track", track: "off" };
  const REPEAT_LABELS: Record<RepeatMode, string> = { off: "Repeat off", list: "Repeat all", track: "Repeat one" };

  const send = (action: string, value?: unknown) => void activityAction(activityId, action, value);
  const sideButton =
    "grid size-10 place-items-center rounded-full text-island-text transition-[background-color,transform] duration-150 enabled:hover:bg-island-hover enabled:active:scale-90 disabled:opacity-30";
  const modeButton =
    "relative grid size-9 place-items-center rounded-full transition-[background-color,color] duration-150 hover:bg-island-hover";
</script>

<div class="flex items-center justify-center gap-2">
  {#if media.shuffle !== null}
    <button
      class="{modeButton} {media.shuffle ? 'text-accent' : 'text-island-subtle'}"
      type="button"
      aria-label="Shuffle"
      aria-pressed={media.shuffle}
      onclick={() => send("shuffle", !media.shuffle)}
    >
      <Shuffle class="size-4" />
      {#if media.shuffle}<span class="absolute bottom-1 size-1 rounded-full bg-accent"></span>{/if}
    </button>
  {/if}
  <button class={sideButton} type="button" aria-label="Previous" disabled={!media.canGoPrevious} onclick={() => send("prev")}>
    <SkipBack class="size-5" fill="currentColor" />
  </button>
  <button
    class="grid size-12 place-items-center rounded-full bg-island-text text-island transition-transform duration-150 hover:scale-105 active:scale-95"
    type="button"
    aria-label={media.playing ? "Pause" : "Play"}
    onclick={() => send("toggle")}
  >
    {#if media.playing}
      <Pause class="size-5" fill="currentColor" />
    {:else}
      <Play class="ml-0.5 size-5" fill="currentColor" />
    {/if}
  </button>
  <button class={sideButton} type="button" aria-label="Next" disabled={!media.canGoNext} onclick={() => send("next")}>
    <SkipForward class="size-5" fill="currentColor" />
  </button>
  {#if media.repeat !== null}
    <button
      class="{modeButton} {media.repeat === 'off' ? 'text-island-subtle' : 'text-accent'}"
      type="button"
      aria-label={REPEAT_LABELS[media.repeat]}
      title={REPEAT_LABELS[media.repeat]}
      onclick={() => media.repeat && send("repeat", NEXT_REPEAT[media.repeat])}
    >
      {#if media.repeat === "track"}
        <Repeat1 class="size-4" />
      {:else}
        <Repeat class="size-4" />
      {/if}
      {#if media.repeat !== "off"}<span class="absolute bottom-1 size-1 rounded-full bg-accent"></span>{/if}
    </button>
  {/if}
</div>
