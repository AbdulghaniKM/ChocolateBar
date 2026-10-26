<script lang="ts">
  import Glyph from "../../shell/Glyph.svelte";
  import type { Activity } from "../../lib/contracts";
  import { LOW_BATTERY_PERCENT, type BluetoothView } from "./types";

  let { activity }: { activity: Activity } = $props();

  const device = $derived(activity.expanded as BluetoothView);
  const isLow = $derived(device.battery !== null && device.battery <= LOW_BATTERY_PERCENT);
</script>

<div class="flex h-full w-full min-w-0 items-center gap-2.5 px-4">
  <span class="relative text-island-text">
    <Glyph glyph={activity.compact.icon} class="size-[1.125rem]" />
    <span
      class="absolute -right-0.5 -bottom-0.5 size-[0.4375rem] rounded-full ring-2 ring-island transition-colors duration-300
        {device.connected ? 'bg-info' : 'bg-island-subtle'}"
    ></span>
  </span>
  <div class="flex min-w-0 flex-1 flex-col leading-tight">
    <span class="truncate text-[0.8125rem] font-medium text-island-text">{device.name}</span>
    {#if activity.compact.subtitle}
      <span class="truncate text-[0.6875rem] text-island-muted">{activity.compact.subtitle}</span>
    {/if}
  </div>
  {#if device.battery !== null}
    <span
      class="rounded-pill bg-island-raised px-2 py-0.5 text-[0.6875rem] font-semibold tabular-nums {isLow ? 'text-danger' : 'text-island-text'}"
    >
      {device.battery}%
    </span>
  {/if}
</div>
