<script lang="ts">
  import { settingsState } from "../../lib/settings.svelte";

  let { source, label, hint }: { source: string; label: string; hint?: string } = $props();

  const enabled = $derived(settingsState.value.sources[source] !== false);
</script>

<label class="flex items-start gap-3 text-sm">
  <input
    type="checkbox"
    class="mt-0.5 accent-accent"
    checked={enabled}
    onchange={(event) => settingsState.update({ sources: { [source]: event.currentTarget.checked } })}
  />
  <span class="flex flex-col gap-0.5">
    <span>{label}</span>
    {#if hint}
      <span class="text-xs text-island-muted">{hint}</span>
    {/if}
  </span>
</label>
<p class="text-xs text-island-muted">Changing this takes effect after restarting ChocolateBar.</p>
