<script lang="ts">
  import Glyph from "../../shell/Glyph.svelte";
  import type { Activity } from "../../lib/contracts";
  import type { WeatherView } from "./types";

  let { activity }: { activity: Activity } = $props();

  const weather = $derived(activity.expanded as WeatherView);
  const isAlert = $derived(weather.alert !== null);
</script>

<div class="flex h-full w-full min-w-0 items-center gap-2.5 px-3 text-sm">
  <span class="text-accent transition-transform duration-300 ease-island" class:scale-110={isAlert}>
    <Glyph glyph={activity.compact.icon} class="size-[1.125rem]" />
  </span>
  <span class="font-semibold tabular-nums">{activity.compact.title}</span>
  <span
    class="min-w-0 truncate text-xs {isAlert ? 'font-medium text-[var(--weather-accent)]' : 'text-island-muted'}"
    style:--weather-accent={activity.compact.accent}
  >
    {activity.compact.subtitle}
  </span>
</div>
