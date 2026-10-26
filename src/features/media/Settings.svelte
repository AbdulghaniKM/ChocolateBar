<script lang="ts">
  import { settingsState } from "../../lib/settings.svelte";
</script>

<section class="flex flex-col gap-4">
  <h2 class="text-lg font-medium">Media</h2>
  <label class="flex items-center gap-3 text-sm">
    <input
      type="checkbox"
      class="accent-accent"
      checked={settingsState.value.media.lyrics}
      onchange={(event) => settingsState.update({ media: { lyrics: event.currentTarget.checked } })}
    />
    Show synced lyrics (LRCLIB)
  </label>
  <label class="flex items-center gap-3 text-sm">
    <input
      type="checkbox"
      class="accent-accent"
      checked={settingsState.value.media.visualizer}
      onchange={(event) => settingsState.update({ media: { visualizer: event.currentTarget.checked } })}
    />
    Music visualizer bars
  </label>
</section>
