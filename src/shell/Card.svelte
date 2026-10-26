<script lang="ts">
  import ChevronLeft from "@lucide/svelte/icons/chevron-left";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import LayoutGrid from "@lucide/svelte/icons/layout-grid";
  import type { Activity } from "../lib/contracts";
  import { island } from "../lib/island.svelte";
  import { registry } from "../features/registry";
  import ActionBar from "./ActionBar.svelte";
  import Glyph from "./Glyph.svelte";
  import { cardLabel } from "./cardMeta";

  let { activity }: { activity: Activity } = $props();

  const Expanded = $derived(registry[activity.kind].Expanded);
  const count = $derived(island.ring.length);
  const index = $derived(island.ring.findIndex((entry) => entry?.id === activity.id));
  const headerButton =
    "grid size-7 place-items-center rounded-full text-island-muted transition-colors hover:bg-island-hover hover:text-island-text";
</script>

<div class="flex h-full w-full flex-col">
  <header class="flex h-11 shrink-0 items-center gap-2 pr-2 pl-4">
    <span class="grid size-5 place-items-center" style:color={activity.compact.accent ?? "var(--color-accent)"}>
      <Glyph glyph={activity.compact.icon} class="size-3.5" strokeWidth={2.25} />
    </span>
    <span class="eyebrow min-w-0 truncate">{cardLabel(activity)}</span>
    <div class="ml-auto flex items-center gap-0.5">
      {#if count > 1}
        <button class={headerButton} type="button" aria-label="Previous activity" onclick={() => island.cycle(-1)}>
          <ChevronLeft class="size-4" />
        </button>
        <span class="min-w-[2.25rem] text-center text-[0.6875rem] text-island-subtle tabular-nums" aria-live="polite">
          {index + 1} / {count}
        </span>
        <button class={headerButton} type="button" aria-label="Next activity" onclick={() => island.cycle(1)}>
          <ChevronRight class="size-4" />
        </button>
      {/if}
      <button class={headerButton} type="button" aria-label="Shortcuts and clipboard" title="Shortcuts and clipboard" onclick={() => island.openHub()}>
        <LayoutGrid class="size-4" />
      </button>
      <button class={headerButton} type="button" aria-label="Collapse" onclick={() => island.setExpanded(false)}>
        <ChevronDown class="size-4" />
      </button>
    </div>
  </header>
  <div class="min-h-0 flex-1 overflow-hidden">
    <Expanded {activity} />
  </div>
  <ActionBar activityId={activity.id} actions={activity.actions} />
</div>
