<script lang="ts">
  import type { Activity } from "../../lib/contracts";
  import { PLAYING_ACTIVITY_ID } from "./types";

  let { activity }: { activity: Activity } = $props();

  const isPlaying = $derived(activity.id === PLAYING_ACTIVITY_ID);
</script>

<div class="flex h-full w-full min-w-0 items-center gap-2.5 px-3 text-sm">
  <span class="relative flex size-6 shrink-0 items-center justify-center text-lg leading-none">
    {activity.compact.icon}
    {#if isPlaying}
      <span class="absolute -right-0.5 -bottom-0.5 size-2 animate-pulse rounded-full bg-success"></span>
    {/if}
  </span>
  <span class="min-w-0 truncate font-medium">{activity.compact.title}</span>
  <span class="ml-auto shrink-0 truncate text-xs text-island-muted">{activity.compact.subtitle}</span>
</div>
