<script lang="ts">
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";
  import { island } from "../lib/island.svelte";
  import { settingsState } from "../lib/settings.svelte";
  import { trackHitRegions } from "../lib/hitRegions";
  import { COMPACT_SIZE, alignmentClasses, cardSize, syncWindowSize, toRem } from "../lib/geometry";
  import Pill from "./Pill.svelte";
  import IdlePill from "./IdlePill.svelte";
  import Card from "./Card.svelte";
  import IdleCard from "./IdleCard.svelte";

  const WHEEL_COOLDOWN_MS = 250;

  let surface = $state<HTMLDivElement>();
  let lastWheelAt = 0;

  const position = $derived(settingsState.value.general.position);
  const size = $derived(toRem(island.expanded ? cardSize(island.focused) : COMPACT_SIZE));

  onMount(() => {
    const stopListening = island.start();
    const stopSizing = syncWindowSize();
    return () => {
      stopSizing();
      void stopListening.then((stop) => stop());
    };
  });

  $effect(() => {
    if (!surface) return;
    return trackHitRegions([surface]);
  });

  function cycleOnWheel(event: WheelEvent) {
    const now = performance.now();
    if (now - lastWheelAt < WHEEL_COOLDOWN_MS) return;
    lastWheelAt = now;
    island.cycle(event.deltaY > 0 ? 1 : -1);
  }

  function collapseOnRightClick(event: MouseEvent) {
    event.preventDefault();
    if (island.expanded) island.setExpanded(false);
  }

  function collapseOnEscape(event: KeyboardEvent) {
    if (event.key === "Escape") island.setExpanded(false);
  }
</script>

<div class="fixed inset-0 flex p-4 {alignmentClasses(position)}" class:invisible={island.hidden}>
  <div
    bind:this={surface}
    role="presentation"
    class="island-surface relative overflow-hidden text-island-text transition-[width,height,border-radius] duration-(--island-duration) ease-island
      {island.expanded ? 'rounded-island' : 'rounded-[1.25rem]'}"
    style:width={size.width}
    style:height={size.height}
    onwheel={cycleOnWheel}
    onkeydown={collapseOnEscape}
    oncontextmenu={collapseOnRightClick}
    onpointerenter={() => island.cancelCollapse()}
    onpointerleave={() => island.scheduleCollapse()}
  >
    {#if island.expanded}
      <div class="absolute inset-0" in:fade={{ duration: 180, delay: 140 }} out:fade={{ duration: 90 }}>
        {#if island.focused}
          <Card activity={island.focused} />
        {:else}
          <IdleCard />
        {/if}
      </div>
    {:else}
      <button
        class="absolute inset-0 rounded-[inherit] text-left focus-visible:outline-offset-[-2px]"
        type="button"
        aria-label="Expand island"
        in:fade={{ duration: 180, delay: 140 }}
        out:fade={{ duration: 90 }}
        onclick={() => island.setExpanded(true)}
      >
        {#if island.focused}
          <Pill activity={island.focused} />
        {:else}
          <IdlePill />
        {/if}
      </button>
    {/if}
  </div>
</div>
