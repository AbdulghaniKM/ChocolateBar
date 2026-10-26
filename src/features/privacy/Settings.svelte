<script lang="ts">
  import { settingsState } from "../../lib/settings.svelte";
</script>

<section class="flex flex-col gap-4">
  <h2 class="text-lg font-medium">Privacy</h2>
  <p class="text-sm text-island-muted">
    Shows an indicator whenever an app is using your microphone or camera.
  </p>
  <label class="flex items-center gap-3 text-sm">
    <span class="w-[10rem]">Mute microphone hotkey</span>
    <input
      class="rounded-lg bg-island-raised px-3 py-1"
      value={settingsState.value.hotkeys.micMute}
      onchange={(event) => settingsState.update({ hotkeys: { micMute: event.currentTarget.value } })}
    />
  </label>
</section>
