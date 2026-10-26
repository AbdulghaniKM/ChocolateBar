<script lang="ts">
  import Settings from "@lucide/svelte/icons/settings";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import { useClock } from "../lib/clock.svelte";
  import { island } from "../lib/island.svelte";
  import { openSettings } from "../lib/ipc";
  import { registry } from "../features/registry";

  const clock = useClock();
  const launchers = Object.values(registry).flatMap((entry) => (entry.Launcher ? [entry.Launcher] : []));
  const headerButton =
    "grid size-7 place-items-center rounded-full text-island-muted transition-colors hover:bg-island-hover hover:text-island-text";
</script>

<div class="flex h-full w-full flex-col">
  <header class="flex h-11 shrink-0 items-center gap-2 pr-2 pl-4">
    <span class="eyebrow">ChocolateBar</span>
    <div class="ml-auto flex items-center gap-0.5">
      <button class={headerButton} type="button" aria-label="Open settings" onclick={() => void openSettings()}>
        <Settings class="size-4" />
      </button>
      <button class={headerButton} type="button" aria-label="Collapse" onclick={() => island.setExpanded(false)}>
        <ChevronDown class="size-4" />
      </button>
    </div>
  </header>
  <div class="flex flex-1 flex-col items-center justify-center gap-1 pb-2">
    <p class="font-display text-[3.25rem] leading-none font-semibold tracking-tight tabular-nums">{clock.time}</p>
    <p class="text-[0.8125rem] text-island-muted">{clock.date}</p>
  </div>
  {#if launchers.length > 0}
    <div class="flex items-center justify-center gap-2 px-4 pb-4">
      {#each launchers as Launcher}
        <Launcher />
      {/each}
    </div>
  {/if}
</div>
