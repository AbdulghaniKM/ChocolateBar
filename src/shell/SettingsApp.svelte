<script lang="ts">
  import { onMount } from "svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import { onSettingsTab } from "../lib/ipc";
  import appIcon from "../../app-icon.svg";
  import { ACTIVITY_KINDS } from "../lib/contracts";
  import { settingsState } from "../lib/settings.svelte";
  import { registry } from "../features/registry";
  import Panel from "./settings/Panel.svelte";
  import CardOrder from "./settings/CardOrder.svelte";
  import PositionPicker from "./settings/PositionPicker.svelte";
  import ShortcutsEditor from "./settings/ShortcutsEditor.svelte";
  import { GENERAL_TAB, SHORTCUTS_TAB, TAB_META } from "./settings/tabs";

  const featureTabs = ACTIVITY_KINDS.filter((kind) => registry[kind].Settings);
  const toggleableSources = [
    "media", "visualizer", "osd", "privacy", "buds", "bluetooth", "battery", "keyboard", "download",
    "network", "claude", "notification", "ocr", "shelf", "sports", "timer", "weather", "game", "clipboard",
    "launcher",
  ];
  const topTabs = [GENERAL_TAB, SHORTCUTS_TAB];
  const allTabs = [...topTabs, ...featureTabs];
  const tabFromHash = () => {
    const requested = location.hash.split("/")[1];
    return requested && allTabs.includes(requested as never) ? requested : GENERAL_TAB;
  };

  let activeTab = $state<string>(tabFromHash());
  let version = $state("");
  const ActiveSettings = $derived(
    topTabs.includes(activeTab) ? undefined : registry[activeTab as (typeof ACTIVITY_KINDS)[number]].Settings,
  );

  onMount(() => {
    const stopListening = settingsState.start();
    void getVersion().then((value) => (version = value));
    const stopTabEvents = onSettingsTab((tab) => {
      if (allTabs.includes(tab as never)) activeTab = tab;
    });
    return () => {
      void stopListening.then((stop) => stop());
      void stopTabEvents.then((stop) => stop());
    };
  });

  const isSourceEnabled = (name: string) => settingsState.value.sources[name] !== false;
  const tabLabel = (tab: string) => TAB_META[tab as keyof typeof TAB_META]?.label ?? tab;
</script>

<div class="settings-app flex h-full bg-[#140e0b] text-island-text">
  <nav class="flex w-[14.5rem] shrink-0 flex-col border-r border-island-border/70 bg-island">
    <div class="flex items-center gap-3 px-5 pt-6 pb-5">
      <img src={appIcon} alt="" class="size-9 rounded-[0.625rem]" />
      <div class="flex flex-col leading-tight">
        <span class="font-display text-[0.9375rem] font-semibold">ChocolateBar</span>
        <span class="text-[0.6875rem] text-island-subtle">{version ? `Version ${version}` : " "}</span>
      </div>
    </div>
    <div class="flex min-h-0 flex-1 flex-col gap-0.5 overflow-y-auto px-3 pb-4">
      {#each allTabs as tab, index (tab)}
        {@const Icon = TAB_META[tab as keyof typeof TAB_META].icon}
        {#if index === topTabs.length}
          <span class="eyebrow mt-4 mb-1.5 px-3">Activities</span>
        {/if}
        <button
          class="flex h-9 items-center gap-3 rounded-lg px-3 text-left text-[0.8125rem] transition-colors
            {activeTab === tab
            ? 'bg-island-raised font-medium text-island-text shadow-[inset_2px_0_0_var(--color-accent)]'
            : 'text-island-muted hover:bg-island-raised/60 hover:text-island-text'}"
          type="button"
          aria-current={activeTab === tab ? "page" : undefined}
          onclick={() => (activeTab = tab)}
        >
          <Icon class="size-4 shrink-0 {activeTab === tab ? 'text-accent' : ''}" />
          <span class="truncate">{tabLabel(tab)}</span>
        </button>
      {/each}
    </div>
  </nav>

  <main class="min-w-0 flex-1 overflow-y-auto">
    <div class="mx-auto flex max-w-[44rem] flex-col gap-5 px-8 py-8">
      <h1 class="font-display text-[1.625rem] font-semibold tracking-tight">{tabLabel(activeTab)}</h1>
      {#if activeTab === SHORTCUTS_TAB}
        <ShortcutsEditor />
      {:else if ActiveSettings}
        <div class="feature-settings rounded-2xl border border-island-border bg-island-raised/60 p-5">
          <ActiveSettings />
        </div>
      {:else}
        <Panel title="Position" description="Pick a spot for the island, then nudge it into place.">
          <PositionPicker />
        </Panel>

        <Panel title="Cards" description="Choose what the island rests on and the order cards come in.">
          <CardOrder />
        </Panel>

        <Panel title="Behavior">
          <label class="flex items-center justify-between gap-4 text-sm">
            <span class="flex flex-col gap-0.5">
              Hide in fullscreen apps
              <span class="text-xs text-island-muted">Games and videos stay uncovered. Critical alerts still show.</span>
            </span>
            <input
              type="checkbox"
              checked={settingsState.value.general.hideInFullscreen}
              onchange={(event) => settingsState.update({ general: { hideInFullscreen: event.currentTarget.checked } })}
            />
          </label>
          <label class="flex items-center justify-between gap-4 text-sm">
            Start with Windows
            <input
              type="checkbox"
              checked={settingsState.value.general.autostart}
              onchange={(event) => settingsState.update({ general: { autostart: event.currentTarget.checked } })}
            />
          </label>
        </Panel>

        <Panel title="Hotkeys">
          <label class="flex items-center justify-between gap-4 text-sm">
            Mute microphone
            <input
              class="w-[11rem]"
              value={settingsState.value.hotkeys.micMute}
              onchange={(event) => settingsState.update({ hotkeys: { micMute: event.currentTarget.value } })}
            />
          </label>
          <label class="flex items-center justify-between gap-4 text-sm">
            Capture text (OCR)
            <input
              class="w-[11rem]"
              value={settingsState.value.hotkeys.ocr}
              onchange={(event) => settingsState.update({ hotkeys: { ocr: event.currentTarget.value } })}
            />
          </label>
        </Panel>

        <Panel title="Activities" description="Turn sources on or off. Changes apply after restarting ChocolateBar.">
          <div class="grid grid-cols-2 gap-x-6 gap-y-3">
            {#each toggleableSources as name (name)}
              <label class="flex items-center justify-between gap-3 text-sm capitalize">
                {name}
                <input
                  type="checkbox"
                  checked={isSourceEnabled(name)}
                  onchange={(event) => settingsState.update({ sources: { [name]: event.currentTarget.checked } })}
                />
              </label>
            {/each}
          </div>
        </Panel>

        <p class="text-xs text-island-subtle">
          Updates install automatically. Use the tray menu to check now. Right-click the island to collapse it.
        </p>
      {/if}
    </div>
  </main>
</div>

<style>
  .feature-settings :global(section > h2:first-child) {
    display: none;
  }
</style>
