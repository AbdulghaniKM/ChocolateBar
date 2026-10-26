<script lang="ts">
  import type { Activity } from "../../lib/contracts";
  import type { WeatherView } from "./types";

  let { activity }: { activity: Activity } = $props();

  const weather = $derived(activity.expanded as WeatherView);

  const details = $derived(
    [
      { label: "Feels like", value: weather.feelsLike === null ? null : `${weather.feelsLike}°` },
      { label: "Humidity", value: weather.humidityPercent === null ? null : `${weather.humidityPercent}%` },
      { label: "Wind", value: weather.wind === null ? null : `${weather.wind} ${weather.speedUnit}` },
      { label: "Gusts", value: `${weather.gust} ${weather.speedUnit}` },
      { label: "Rain", value: `${weather.precipitationMm.toFixed(1)} mm` },
      {
        label: "High / Low",
        value: weather.high === null || weather.low === null ? null : `${weather.high}° / ${weather.low}°`,
      },
    ].filter((detail) => detail.value !== null),
  );
</script>

<div class="flex h-full flex-col gap-4 p-5">
  {#if weather.alert}
    <div
      class="rounded-xl border px-4 py-2 text-sm font-medium"
      style:border-color={activity.compact.accent}
      style:color={activity.compact.accent}
      style:background-color="color-mix(in srgb, {activity.compact.accent ?? 'currentColor'} 14%, transparent)"
    >
      {weather.alert}
    </div>
  {/if}

  <div class="flex items-center gap-5">
    <span class="text-6xl leading-none">{weather.icon}</span>
    <div class="flex min-w-0 flex-col">
      <p class="text-5xl font-light tabular-nums">
        {weather.temperature}<span class="text-2xl text-island-muted">{weather.temperatureUnit}</span>
      </p>
      <p class="truncate text-sm text-island-muted">
        {weather.description}{weather.city ? ` · ${weather.city}` : ""}
      </p>
    </div>
  </div>

  <dl class="grid grid-cols-3 gap-2">
    {#each details as detail (detail.label)}
      <div class="flex flex-col rounded-xl bg-island-raised px-3 py-2">
        <dt class="text-[0.6875rem] tracking-wide text-island-muted uppercase">{detail.label}</dt>
        <dd class="text-sm font-medium tabular-nums">{detail.value}</dd>
      </div>
    {/each}
  </dl>
</div>
