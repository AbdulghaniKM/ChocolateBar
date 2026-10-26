<script lang="ts">
  import ArrowDown from "@lucide/svelte/icons/arrow-down";
  import ArrowUp from "@lucide/svelte/icons/arrow-up";
  import Clock from "@lucide/svelte/icons/clock";
  import RotateCcw from "@lucide/svelte/icons/rotate-ccw";
  import { CLOCK_CARD, DEFAULT_CARD_ORDER, type ActivityKind } from "../../lib/contracts";
  import { settingsState } from "../../lib/settings.svelte";
  import { TAB_META } from "./tabs";

  const DEFAULT_CHOICES = [CLOCK_CARD, "network", "weather", "battery", "media"];

  const general = $derived(settingsState.value.general);
  const order = $derived(completeOrder(general.cardOrder));
  const clockFirst = $derived(general.defaultCard === CLOCK_CARD);

  function completeOrder(saved: string[]): string[] {
    const known = saved.filter((kind) => kind in TAB_META && kind !== CLOCK_CARD);
    return [...known, ...DEFAULT_CARD_ORDER.filter((kind) => !known.includes(kind))];
  }

  const labelOf = (kind: string) => (kind === CLOCK_CARD ? "Clock" : (TAB_META[kind as ActivityKind]?.label ?? kind));

  function move(index: number, offset: -1 | 1) {
    const next = [...order];
    const target = index + offset;
    if (target < 0 || target >= next.length) return;
    [next[index], next[target]] = [next[target], next[index]];
    void settingsState.update({ general: { cardOrder: next } });
  }

  const arrowButton =
    "grid size-7 place-items-center rounded-md text-island-muted transition-colors hover:bg-island-hover hover:text-island-text disabled:pointer-events-none disabled:opacity-25";
</script>

<div class="flex flex-col gap-4">
  <label class="flex items-center justify-between gap-4 text-sm">
    <span class="flex flex-col gap-0.5">
      Default card
      <span class="text-xs text-island-muted">Shown when nothing live is happening. Music, timers and alerts still take over.</span>
    </span>
    <select
      class="w-[11rem]"
      value={general.defaultCard}
      onchange={(event) => settingsState.update({ general: { defaultCard: event.currentTarget.value } })}
    >
      {#each DEFAULT_CHOICES as choice (choice)}
        <option value={choice}>{labelOf(choice)}</option>
      {/each}
    </select>
  </label>

  <div class="flex flex-col gap-2">
    <div class="flex items-center justify-between">
      <span class="flex flex-col gap-0.5 text-sm">
        Card order
        <span class="text-xs text-island-muted">The order you flip through cards with the scroll wheel or the header arrows.</span>
      </span>
      <button
        class="inline-flex h-8 shrink-0 items-center gap-1.5 rounded-pill bg-island px-3 text-xs text-island-muted transition-colors hover:text-island-text"
        type="button"
        onclick={() => settingsState.update({ general: { cardOrder: [...DEFAULT_CARD_ORDER] } })}
      >
        <RotateCcw class="size-3.5" /> Reset
      </button>
    </div>
    <ol class="flex flex-col gap-1 rounded-xl border border-island-border bg-island p-1.5">
      {#if clockFirst}
        <li class="flex h-9 items-center gap-3 rounded-lg bg-accent/10 px-2.5 text-sm">
          <span class="w-5 text-right text-xs text-island-subtle tabular-nums">1</span>
          <Clock class="size-4 text-accent" />
          <span class="flex-1">Clock</span>
          <span class="text-xs text-island-subtle">Default card</span>
        </li>
      {/if}
      {#each order as kind, index (kind)}
        {@const Icon = TAB_META[kind as ActivityKind]?.icon}
        <li class="flex h-9 items-center gap-3 rounded-lg px-2.5 text-sm transition-colors hover:bg-island-raised">
          <span class="w-5 text-right text-xs text-island-subtle tabular-nums">{index + (clockFirst ? 2 : 1)}</span>
          {#if Icon}<Icon class="size-4 text-island-muted" />{/if}
          <span class="flex-1">{labelOf(kind)}</span>
          {#if general.defaultCard === kind}
            <span class="text-xs text-accent">Default card</span>
          {/if}
          <button class={arrowButton} type="button" aria-label="Move {labelOf(kind)} up" disabled={index === 0} onclick={() => move(index, -1)}>
            <ArrowUp class="size-3.5" />
          </button>
          <button
            class={arrowButton}
            type="button"
            aria-label="Move {labelOf(kind)} down"
            disabled={index === order.length - 1}
            onclick={() => move(index, 1)}
          >
            <ArrowDown class="size-3.5" />
          </button>
        </li>
      {/each}
    </ol>
  </div>
</div>
