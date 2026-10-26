<script lang="ts">
  import ArrowLeft from "@lucide/svelte/icons/arrow-left";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ClipboardList from "@lucide/svelte/icons/clipboard-list";
  import Rocket from "@lucide/svelte/icons/rocket";
  import Settings from "@lucide/svelte/icons/settings";
  import Timer from "@lucide/svelte/icons/timer";
  import { island, type HubTab } from "../../lib/island.svelte";
  import { openSettings } from "../../lib/ipc";
  import ClipboardTab from "./ClipboardTab.svelte";
  import ShortcutsTab from "./ShortcutsTab.svelte";
  import TimerTab from "./TimerTab.svelte";

  const TABS: { id: HubTab; label: string; icon: typeof Rocket }[] = [
    { id: "shortcuts", label: "Shortcuts", icon: Rocket },
    { id: "clipboard", label: "Clipboard", icon: ClipboardList },
    { id: "timer", label: "Clock", icon: Timer },
  ];
  const headerButton =
    "grid size-7 place-items-center rounded-full text-island-muted transition-colors hover:bg-island-hover hover:text-island-text";
</script>

<div class="flex h-full w-full flex-col">
  <header class="flex h-11 shrink-0 items-center gap-1 pr-2 pl-2.5">
    {#if island.focused}
      <button class={headerButton} type="button" aria-label="Back to {island.focused.compact.title}" onclick={() => island.closeHub()}>
        <ArrowLeft class="size-4" />
      </button>
    {/if}
    <div class="flex items-center gap-0.5 rounded-pill bg-island-raised/70 p-0.5" role="tablist" aria-label="Hub">
      {#each TABS as tab (tab.id)}
        {@const Icon = tab.icon}
        {@const active = island.hubTab === tab.id}
        <button
          class="inline-flex h-7 items-center gap-1.5 rounded-pill px-3 text-[0.75rem] font-medium transition-colors duration-150
            {active ? 'bg-island-text text-island' : 'text-island-muted hover:text-island-text'}"
          type="button"
          role="tab"
          aria-selected={active}
          onclick={() => (island.hubTab = tab.id)}
        >
          <Icon class="size-3.5" />
          {tab.label}
        </button>
      {/each}
    </div>
    <div class="ml-auto flex items-center gap-0.5">
      <button class={headerButton} type="button" aria-label="Open settings" onclick={() => void openSettings()}>
        <Settings class="size-4" />
      </button>
      <button class={headerButton} type="button" aria-label="Collapse" onclick={() => island.setExpanded(false)}>
        <ChevronDown class="size-4" />
      </button>
    </div>
  </header>
  <div class="min-h-0 flex-1 px-4 pb-4" role="tabpanel">
    {#if island.hubTab === "shortcuts"}
      <ShortcutsTab />
    {:else if island.hubTab === "clipboard"}
      <ClipboardTab />
    {:else}
      <TimerTab />
    {/if}
  </div>
</div>
