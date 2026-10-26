<script lang="ts">
  import type { Activity } from "../../lib/contracts";
  import BatteryGauge from "./BatteryGauge.svelte";
  import type { BatteryView } from "./types";

  let { activity }: { activity: Activity } = $props();

  const battery = $derived(activity.expanded as BatteryView);
  const statusText = $derived(describeStatus(battery));
  const remainingText = $derived(describeRemaining(battery.minutesRemaining));

  function describeStatus(view: BatteryView) {
    if (view.event === "low" || view.low) return "Battery low - plug in soon";
    if (view.charging) return "Charging";
    if (view.full) return "Fully charged";
    if (view.pluggedIn) return "Plugged in";
    return "On battery";
  }

  function describeRemaining(minutes: number | null) {
    if (minutes === null) return null;
    return `${Math.floor(minutes / 60)}h ${String(minutes % 60).padStart(2, "0")}m remaining`;
  }
</script>

<div class="flex h-full flex-col items-center justify-center gap-4 px-6">
  <BatteryGauge percent={battery.percent} charging={battery.charging} low={battery.low} large />
  <div class="flex flex-col items-center gap-1">
    <span class="text-[2.25rem] leading-none font-semibold tabular-nums text-island-text">
      {battery.percent}<span class="text-[1.25rem] text-island-muted">%</span>
    </span>
    <span
      class="rounded-pill border border-white/10 bg-white/5 px-3 py-0.5 text-[0.75rem] backdrop-blur transition-colors duration-300
        {battery.low ? 'text-danger' : battery.charging ? 'text-success' : 'text-island-muted'}"
    >
      {statusText}
    </span>
    {#if remainingText}
      <span class="text-[0.75rem] text-island-muted">{remainingText}</span>
    {/if}
  </div>
</div>
