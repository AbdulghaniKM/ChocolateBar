<script lang="ts">
  import { settingsState } from "../../lib/settings.svelte";

  const MAX_MINUTES = 240;

  const fields = [
    { key: "workMin", label: "Focus length" },
    { key: "shortMin", label: "Short break" },
    { key: "longMin", label: "Long break" },
  ] as const;

  function saveMinutes(key: (typeof fields)[number]["key"], input: HTMLInputElement) {
    const minutes = Math.min(MAX_MINUTES, Math.max(1, Math.round(Number(input.value) || 1)));
    input.value = String(minutes);
    void settingsState.update({ timer: { [key]: minutes } });
  }
</script>

<section class="flex flex-col gap-4">
  <h2 class="text-lg font-medium">Timer</h2>
  <p class="text-sm text-island-muted">
    Pomodoro runs focus, then a short break three times, then a long break. Changes apply to the next
    Pomodoro you start.
  </p>
  {#each fields as field (field.key)}
    <label class="flex items-center gap-3 text-sm">
      <span class="w-[8rem]">{field.label}</span>
      <input
        class="w-[5rem] rounded-lg bg-island-raised px-3 py-1 tabular-nums outline-none focus:ring-2 focus:ring-accent"
        type="number"
        min="1"
        max={MAX_MINUTES}
        value={settingsState.value.timer[field.key]}
        onchange={(event) => saveMinutes(field.key, event.currentTarget)}
      />
      <span class="text-island-muted">min</span>
    </label>
  {/each}
</section>
