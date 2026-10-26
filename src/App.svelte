<script lang="ts">
  import { onMount } from "svelte";
  import { settingsState } from "./lib/settings.svelte";
  import Island from "./shell/Island.svelte";
  import SettingsApp from "./shell/SettingsApp.svelte";

  const showsSettings = location.hash === "#settings";

  onMount(() => {
    const stopListening = settingsState.start();
    return () => void stopListening.then((stop) => stop());
  });
</script>

{#if showsSettings}
  <SettingsApp />
{:else}
  <Island />
{/if}
