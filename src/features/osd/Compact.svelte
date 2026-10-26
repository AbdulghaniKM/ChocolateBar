<script lang="ts">
  import type { Activity } from "../../lib/contracts";
  import LevelBar from "./LevelBar.svelte";
  import type { OsdExpanded } from "./types";

  let { activity }: { activity: Activity } = $props();

  const osd = $derived(activity.expanded as OsdExpanded);
</script>

<div class="flex h-full w-full items-center gap-3 px-4">
  <span class="w-[1.25rem] shrink-0 text-center text-base">{activity.compact.icon}</span>
  <div class="min-w-0 flex-1">
    <LevelBar percent={osd.percent} muted={osd.muted} />
  </div>
  <span class="w-[2.5rem] shrink-0 text-right text-[0.8125rem] font-medium tabular-nums text-island-text">
    {osd.muted ? "Mute" : `${osd.percent}%`}
  </span>
</div>
