<script lang="ts">
  import { settingsState } from "../../lib/settings.svelte";
</script>

<section class="flex flex-col gap-4">
  <h2 class="text-lg font-medium">On-screen display</h2>
  <label class="flex items-center gap-3 text-sm">
    <input
      type="checkbox"
      class="accent-accent"
      checked={settingsState.value.osd.replaceVolume}
      onchange={(event) => settingsState.update({ osd: { replaceVolume: event.currentTarget.checked } })}
    />
    Replace the Windows volume overlay (captures volume keys)
  </label>
  <label class="flex items-center gap-3 text-sm">
    <input
      type="checkbox"
      class="accent-accent"
      checked={settingsState.value.osd.brightness}
      onchange={(event) => settingsState.update({ osd: { brightness: event.currentTarget.checked } })}
    />
    Show brightness changes
  </label>
</section>
