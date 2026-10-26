<script lang="ts">
  import { onMount } from "svelte";
  import { ACTIVITY_KINDS } from "../lib/contracts";
  import { settingsState } from "../lib/settings.svelte";
  import { registry } from "../features/registry";

  const GENERAL_TAB = "general";
  const featureTabs = ACTIVITY_KINDS.filter((kind) => registry[kind].Settings);
  const toggleableSources = [
    "media", "visualizer", "osd", "privacy", "buds", "bluetooth", "battery", "keyboard", "download",
    "network", "claude", "notification", "ocr", "shelf", "sports", "timer", "weather", "game",
  ];

  let activeTab = $state<string>(GENERAL_TAB);
  const ActiveSettings = $derived(
    activeTab === GENERAL_TAB ? undefined : registry[activeTab as (typeof ACTIVITY_KINDS)[number]].Settings,
  );

  onMount(() => {
    const stopListening = settingsState.start();
    return () => void stopListening.then((stop) => stop());
  });

  const isSourceEnabled = (name: string) => settingsState.value.sources[name] !== false;
</script>

<div class="flex h-full bg-island text-island-text">
  <nav class="flex w-[11rem] shrink-0 flex-col gap-1 overflow-y-auto border-r border-island-border p-3">
    {#each [GENERAL_TAB, ...featureTabs] as tab (tab)}
      <button
        class="rounded-lg px-3 py-2 text-left text-sm capitalize {activeTab === tab
          ? 'bg-island-raised'
          : 'text-island-muted hover:bg-island-raised'}"
        type="button"
        onclick={() => (activeTab = tab)}
      >
        {tab}
      </button>
    {/each}
  </nav>

  <main class="min-w-0 flex-1 overflow-y-auto p-6">
    {#if ActiveSettings}
      <ActiveSettings />
    {:else}
      <section class="flex flex-col gap-4">
        <h2 class="text-lg font-medium">General</h2>
        <label class="flex items-center gap-3 text-sm">
          <input
            type="checkbox"
            class="accent-accent"
            checked={settingsState.value.general.hideInFullscreen}
            onchange={(event) =>
              settingsState.update({ general: { hideInFullscreen: event.currentTarget.checked } })}
          />
          Hide in fullscreen apps
        </label>
        <label class="flex items-center gap-3 text-sm">
          <input
            type="checkbox"
            class="accent-accent"
            checked={settingsState.value.general.autostart}
            onchange={(event) =>
              settingsState.update({ general: { autostart: event.currentTarget.checked } })}
          />
          Start with Windows
        </label>

        <h2 class="mt-4 text-lg font-medium">Hotkeys</h2>
        <label class="flex items-center gap-3 text-sm">
          <span class="w-[8rem]">Mute microphone</span>
          <input
            class="rounded-lg bg-island-raised px-3 py-1"
            value={settingsState.value.hotkeys.micMute}
            onchange={(event) =>
              settingsState.update({ hotkeys: { micMute: event.currentTarget.value } })}
          />
        </label>
        <label class="flex items-center gap-3 text-sm">
          <span class="w-[8rem]">Capture text (OCR)</span>
          <input
            class="rounded-lg bg-island-raised px-3 py-1"
            value={settingsState.value.hotkeys.ocr}
            onchange={(event) => settingsState.update({ hotkeys: { ocr: event.currentTarget.value } })}
          />
        </label>

        <h2 class="mt-4 text-lg font-medium">Sources</h2>
        <p class="text-xs text-island-muted">Changes apply after restarting ChocolateBar.</p>
        <div class="grid grid-cols-2 gap-2">
          {#each toggleableSources as name (name)}
            <label class="flex items-center gap-3 text-sm capitalize">
              <input
                type="checkbox"
                class="accent-accent"
                checked={isSourceEnabled(name)}
                onchange={(event) =>
                  settingsState.update({ sources: { [name]: event.currentTarget.checked } })}
              />
              {name}
            </label>
          {/each}
        </div>
      </section>
    {/if}
  </main>
</div>
