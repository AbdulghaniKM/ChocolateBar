<script lang="ts">
  import Glyph from "../../shell/Glyph.svelte";
  import type { Activity } from "../../lib/contracts";
  import { LOW_BATTERY_PERCENT, type BluetoothView } from "./types";

  let { activity }: { activity: Activity } = $props();

  const device = $derived(activity.expanded as BluetoothView);
  const isLow = $derived(device.battery !== null && device.battery <= LOW_BATTERY_PERCENT);
  const statusText = $derived(describeStatus(device));

  function describeStatus(view: BluetoothView) {
    if (view.event === "lowBattery") return "Battery low";
    if (view.event === "disconnected" || !view.connected) return "Disconnected";
    return "Connected";
  }
</script>

<div class="flex h-full flex-col items-center justify-center gap-5 px-8">
  <div class="relative">
    <div
      class="flex size-[6rem] items-center justify-center tile rounded-[1.75rem] text-info"
    >
      <Glyph glyph={activity.compact.icon} class="size-11" strokeWidth={1.75} />
    </div>
    <span
      class="absolute -right-1 -bottom-1 size-[1.125rem] rounded-full ring-4 ring-island transition-colors duration-300
        {device.connected ? 'bg-info' : 'bg-island-subtle'}"
    ></span>
  </div>
  <div class="flex flex-col items-center gap-1">
    <span class="max-w-[22rem] truncate text-[1.125rem] font-medium text-island-text">{device.name}</span>
    <span class="text-[0.75rem] {device.event === 'lowBattery' ? 'text-danger' : 'text-island-muted'}">{statusText}</span>
  </div>
  {#if device.battery !== null}
    <div class="flex w-full max-w-[18rem] flex-col gap-1.5">
      <div class="flex items-baseline justify-between text-[0.75rem] text-island-muted">
        <span>Battery</span>
        <span class="text-[0.9375rem] font-semibold tabular-nums {isLow ? 'text-danger' : 'text-island-text'}">
          {device.battery}%
        </span>
      </div>
      <div class="h-[0.5rem] overflow-hidden rounded-pill bg-island-raised">
        <div
          class="h-full w-full origin-left rounded-pill transition-transform duration-500 ease-island {isLow ? 'bg-danger' : 'bg-info'}"
          style:transform="scaleX({device.battery / 100})"
        ></div>
      </div>
    </div>
  {/if}
</div>
