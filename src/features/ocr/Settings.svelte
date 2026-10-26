<script lang="ts">
  import { onMount } from "svelte";
  import { settingsState } from "../../lib/settings.svelte";
  import { sourceCommand } from "../../lib/ipc";

  let installedLanguages = $state<string[]>([]);
  let arabicAvailable = $state(false);

  onMount(async () => {
    try {
      const reply = await sourceCommand<{ languages: string[]; arabicAvailable: boolean }>("ocr", "status");
      installedLanguages = reply.languages;
      arabicAvailable = reply.arabicAvailable;
    } catch {
      installedLanguages = [];
    }
  });
</script>

<section class="flex flex-col gap-4">
  <h2 class="text-lg font-medium">Capture text (OCR)</h2>
  <p class="text-sm text-island-muted">
    Press <kbd class="rounded bg-island-raised px-1.5 py-0.5">{settingsState.value.hotkeys.ocr}</kbd>, select an
    area, and the text lands in your clipboard.
  </p>

  <label class="flex items-center gap-3 text-sm">
    <input
      type="checkbox"
      class="accent-accent"
      checked={settingsState.value.ocr.autoClipboard}
      onchange={(event) => settingsState.update({ ocr: { autoClipboard: event.currentTarget.checked } })}
    />
    Recognize every image I copy
  </label>

  <label class="flex items-center gap-3 text-sm">
    <span class="w-[8rem]">Language</span>
    <select
      class="rounded-lg bg-island-raised px-3 py-1"
      value={settingsState.value.ocr.language}
      onchange={(event) => settingsState.update({ ocr: { language: event.currentTarget.value } })}
    >
      <option value="auto">Automatic (profile languages)</option>
      <option value="ar" disabled={!arabicAvailable}>Arabic{arabicAvailable ? "" : " (not installed)"}</option>
    </select>
  </label>

  {#if installedLanguages.length > 0}
    <p class="text-xs text-island-muted">Installed OCR languages: {installedLanguages.join(", ")}</p>
  {/if}
</section>
