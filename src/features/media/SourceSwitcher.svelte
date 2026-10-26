<script lang="ts">
  import AppWindow from "@lucide/svelte/icons/app-window";
  import EighthNotes from "@lucide/svelte/icons/eighth-notes";
  import Globe from "@lucide/svelte/icons/globe";
  import { activityAction } from "../../lib/ipc";
  import type { MediaExpanded, MediaSource } from "./types";

  let { activityId, media }: { activityId: string; media: MediaExpanded } = $props();

  const ICONS = { spotify: EighthNotes, browser: Globe, app: AppWindow } satisfies Record<MediaSource["kind"], unknown>;
</script>

<div class="flex items-center justify-center gap-1.5" role="radiogroup" aria-label="Media source">
  {#each media.sources as source (source.id)}
    {@const Icon = ICONS[source.kind]}
    {@const active = source.id === media.source.id}
    <button
      class="inline-flex h-7 items-center gap-1.5 rounded-pill px-2.5 text-[0.75rem] font-medium transition-colors duration-150
        {active ? 'bg-island-text text-island' : 'bg-island-raised text-island-muted hover:bg-island-hover hover:text-island-text'}"
      type="button"
      role="radio"
      aria-checked={active}
      onclick={() => void activityAction(activityId, "source", source.id)}
    >
      <Icon class="size-3.5" />
      {source.app}
      {#if source.playing && !active}
        <span class="size-1.5 rounded-full bg-success"></span>
      {/if}
    </button>
  {/each}
</div>
