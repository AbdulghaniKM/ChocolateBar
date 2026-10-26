<script lang="ts">
  import Glyph from "../../shell/Glyph.svelte";
  import type { Activity } from "../../lib/contracts";
  import LevelBar from "./LevelBar.svelte";
  import type { OsdExpanded } from "./types";

  let { activity }: { activity: Activity } = $props();

  const osd = $derived(activity.expanded as OsdExpanded);
</script>

<div class="flex h-full w-full items-center gap-3 px-4">
  <Glyph glyph={activity.compact.icon} class="size-[1.125rem] text-island-text" />
  <div class="min-w-0 flex-1">
    <LevelBar percent={osd.percent} muted={osd.muted} />
  </div>
  <span class="w-[2.5rem] shrink-0 text-right text-[0.8125rem] font-medium tabular-nums text-island-text">
    {osd.muted ? "Mute" : `${osd.percent}%`}
  </span>
</div>
