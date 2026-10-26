<script lang="ts">
  import type { Activity } from "../../lib/contracts";
  import BatteryGauge from "./BatteryGauge.svelte";
  import type { BatteryView } from "./types";

  let { activity }: { activity: Activity } = $props();

  const battery = $derived(activity.expanded as BatteryView);
</script>

<div class="flex h-full w-full min-w-0 items-center gap-2.5 px-4">
  <BatteryGauge percent={battery.percent} charging={battery.charging} low={battery.low} />
  <span class="text-[0.875rem] font-semibold tabular-nums text-island-text">{activity.compact.title}</span>
  {#if activity.compact.subtitle}
    <span class="min-w-0 flex-1 truncate text-right text-[0.6875rem] text-island-muted">
      {activity.compact.subtitle}
    </span>
  {/if}
</div>
