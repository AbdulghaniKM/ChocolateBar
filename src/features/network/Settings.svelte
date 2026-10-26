<script lang="ts">
  import { settingsState } from "../../lib/settings.svelte";
  import SourceToggle from "../battery/SourceToggle.svelte";
</script>

<section class="flex flex-col gap-4">
  <h2 class="text-lg font-medium">Network</h2>
  <SourceToggle
    source="network"
    label="Show network status"
    hint="Wi-Fi connect or drop, VPN on or off, and live speed."
  />
  <label class="flex items-center gap-3 text-sm">
    <input
      type="checkbox"
      class="accent-accent"
      checked={settingsState.value.network.showSpeed}
      onchange={(event) => settingsState.update({ network: { showSpeed: event.currentTarget.checked } })}
    />
    Show live upload and download speed
  </label>
</section>
