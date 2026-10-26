<script lang="ts">
  import { settingsState } from "../../lib/settings.svelte";

  const weather = $derived(settingsState.value.weather);
  const usesAutomaticLocation = $derived(weather.lat === null || weather.lon === null);

  const parseCoordinate = (raw: string) => {
    const trimmed = raw.trim();
    return trimmed === "" || Number.isNaN(Number(trimmed)) ? null : Number(trimmed);
  };

  function saveCoordinate(key: "lat" | "lon", input: HTMLInputElement) {
    void settingsState.update({ weather: { [key]: parseCoordinate(input.value) } });
  }

  function saveCity(input: HTMLInputElement) {
    const city = input.value.trim();
    void settingsState.update({ weather: { city: city === "" ? null : city } });
  }

  const inputClasses =
    "rounded-lg bg-island-raised px-3 py-1 outline-none focus:ring-2 focus:ring-accent";
</script>

<section class="flex flex-col gap-4">
  <h2 class="text-lg font-medium">Weather</h2>

  <label class="flex items-center gap-3 text-sm">
    <span class="w-[8rem]">Units</span>
    <select
      class={inputClasses}
      value={weather.units}
      onchange={(event) => settingsState.update({ weather: { units: event.currentTarget.value } })}
    >
      <option value="metric">Metric (&deg;C, km/h)</option>
      <option value="imperial">Imperial (&deg;F, mph)</option>
    </select>
  </label>

  <div class="flex flex-col gap-3">
    <h3 class="text-sm font-medium">Location</h3>
    <p class="text-xs text-island-muted">
      {usesAutomaticLocation
        ? "Detected automatically from your IP address. Enter coordinates to pin a place."
        : "Using the coordinates below."}
    </p>
    <label class="flex items-center gap-3 text-sm">
      <span class="w-[8rem]">City label</span>
      <input class={inputClasses} value={weather.city ?? ""} onchange={(event) => saveCity(event.currentTarget)} />
    </label>
    <label class="flex items-center gap-3 text-sm">
      <span class="w-[8rem]">Latitude</span>
      <input
        class="{inputClasses} w-[9rem] tabular-nums"
        inputmode="decimal"
        value={weather.lat ?? ""}
        onchange={(event) => saveCoordinate("lat", event.currentTarget)}
      />
    </label>
    <label class="flex items-center gap-3 text-sm">
      <span class="w-[8rem]">Longitude</span>
      <input
        class="{inputClasses} w-[9rem] tabular-nums"
        inputmode="decimal"
        value={weather.lon ?? ""}
        onchange={(event) => saveCoordinate("lon", event.currentTarget)}
      />
    </label>
    {#if !usesAutomaticLocation}
      <button
        class="w-fit rounded-lg bg-island-raised px-3 py-1 text-sm transition-colors hover:bg-island-border"
        type="button"
        onclick={() => settingsState.update({ weather: { lat: null, lon: null, city: null } })}
      >
        Use automatic location
      </button>
    {/if}
  </div>
</section>
