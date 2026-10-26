<script lang="ts">
  import type { Activity } from "../../lib/contracts";
  import LevelBar from "./LevelBar.svelte";
  import type { OsdExpanded } from "./types";

  let { activity }: { activity: Activity } = $props();

  const osd = $derived(activity.expanded as OsdExpanded);
  const label = $derived(osd.kind === "brightness" ? "Brightness" : "Volume");
</script>

<div class="flex h-full flex-col items-center justify-center gap-4 px-8">
  <span class="text-5xl">{activity.compact.icon}</span>
  <div class="flex flex-col items-center gap-1">
    <p class="text-3xl font-semibold tabular-nums text-island-text">{osd.muted ? "Muted" : `${osd.percent}%`}</p>
    <p class="text-sm text-island-muted">{label}</p>
  </div>
  <div class="w-full">
    <LevelBar percent={osd.percent} muted={osd.muted} tall />
  </div>
</div>
