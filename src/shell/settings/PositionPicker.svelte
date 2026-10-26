<script lang="ts">
  import RotateCcw from "@lucide/svelte/icons/rotate-ccw";
  import type { HorizontalAnchor, IslandPosition, VerticalAnchor } from "../../lib/contracts";
  import { settingsState } from "../../lib/settings.svelte";

  const VERTICAL: VerticalAnchor[] = ["top", "middle", "bottom"];
  const HORIZONTAL: HorizontalAnchor[] = ["left", "center", "right"];
  const OFFSET_LIMIT = 600;
  const SAVE_DELAY_MS = 120;

  const position = $derived(settingsState.value.general.position);
  let saveTimer: ReturnType<typeof setTimeout> | undefined;

  function save(patch: Partial<IslandPosition>) {
    void settingsState.update({ general: { position: { ...position, ...patch } } });
  }

  function saveSoon(patch: Partial<IslandPosition>) {
    clearTimeout(saveTimer);
    saveTimer = setTimeout(() => save(patch), SAVE_DELAY_MS);
  }

  const label = (vertical: VerticalAnchor, horizontal: HorizontalAnchor) =>
    vertical === "middle" && horizontal === "center" ? "Center of screen" : `${vertical} ${horizontal}`;
</script>

<div class="flex flex-col gap-5 sm:flex-row sm:items-start">
  <div
    class="relative grid aspect-video w-full max-w-[17rem] shrink-0 grid-cols-3 grid-rows-3 gap-1 rounded-xl border border-island-border bg-island p-2"
    role="radiogroup"
    aria-label="Island position"
  >
    <span class="pointer-events-none absolute inset-x-2 bottom-1 h-1 rounded-pill bg-island-raised" aria-hidden="true"></span>
    {#each VERTICAL as vertical (vertical)}
      {#each HORIZONTAL as horizontal (horizontal)}
        {@const active = position.vertical === vertical && position.horizontal === horizontal}
        <button
          class="group flex items-center justify-center rounded-lg transition-colors hover:bg-island-raised
            {vertical === 'top' ? 'items-start' : vertical === 'bottom' ? 'items-end' : ''}
            {horizontal === 'left' ? 'justify-start' : horizontal === 'right' ? 'justify-end' : ''}"
          type="button"
          role="radio"
          aria-checked={active}
          aria-label={label(vertical, horizontal)}
          title={label(vertical, horizontal)}
          onclick={() => save({ vertical, horizontal, offsetX: 0, offsetY: 0 })}
        >
          <span
            class="m-1 h-2 w-7 rounded-pill transition-[background-color,box-shadow] duration-150
              {active ? 'bg-accent shadow-[0_0_0.75rem_rgb(234_166_96/0.5)]' : 'bg-island-border group-hover:bg-island-subtle'}"
          ></span>
        </button>
      {/each}
    {/each}
  </div>

  <div class="flex min-w-0 flex-1 flex-col gap-4">
    <label class="flex flex-col gap-1.5">
      <span class="flex items-center justify-between text-sm">
        Horizontal nudge
        <span class="text-xs text-island-muted tabular-nums">{position.offsetX} px</span>
      </span>
      <input
        type="range"
        min={-OFFSET_LIMIT}
        max={OFFSET_LIMIT}
        step="4"
        value={position.offsetX}
        oninput={(event) => saveSoon({ offsetX: Number(event.currentTarget.value) })}
      />
    </label>
    <label class="flex flex-col gap-1.5">
      <span class="flex items-center justify-between text-sm">
        Vertical nudge
        <span class="text-xs text-island-muted tabular-nums">{position.offsetY} px</span>
      </span>
      <input
        type="range"
        min={-OFFSET_LIMIT}
        max={OFFSET_LIMIT}
        step="4"
        value={position.offsetY}
        oninput={(event) => saveSoon({ offsetY: Number(event.currentTarget.value) })}
      />
    </label>
    <button
      class="inline-flex h-8 w-fit items-center gap-1.5 rounded-pill bg-island-raised px-3 text-xs text-island-muted transition-colors hover:bg-island-hover hover:text-island-text"
      type="button"
      onclick={() => save({ vertical: "bottom", horizontal: "center", offsetX: 0, offsetY: 0 })}
    >
      <RotateCcw class="size-3.5" /> Reset to bottom center
    </button>
  </div>
</div>
