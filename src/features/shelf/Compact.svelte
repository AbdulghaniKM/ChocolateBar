<script lang="ts">
  import Glyph from "../../shell/Glyph.svelte";
  import type { Activity } from "../../lib/contracts";

  let { activity }: { activity: Activity } = $props();

  const hovering = $derived(activity.priority === "critical");
</script>

<div
  class="flex h-full min-w-0 flex-1 items-center gap-2 px-3 text-sm transition-shadow duration-300 {hovering
    ? 'shadow-[inset_0_0_1.5rem_0_rgb(199_161_255/0.45)]'
    : ''}"
>
  <span class="shrink-0 text-berry {hovering ? 'animate-bounce' : ''}"><Glyph glyph={activity.compact.icon} class="size-4" /></span>
  <span class="shrink-0 font-medium">{activity.compact.title}</span>
  {#if activity.compact.subtitle}
    <span class="min-w-0 flex-1 truncate text-island-muted">{activity.compact.subtitle}</span>
  {/if}
</div>
