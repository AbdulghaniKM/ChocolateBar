<script lang="ts">
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";
  import { island } from "../lib/island.svelte";
  import { trackHitRegions } from "../lib/hitRegions";
  import Pill from "./Pill.svelte";
  import IdlePill from "./IdlePill.svelte";
  import Card from "./Card.svelte";
  import IdleCard from "./IdleCard.svelte";

  const WHEEL_COOLDOWN_MS = 250;

  let surface = $state<HTMLDivElement>();
  let lastWheelAt = 0;

  onMount(() => {
    const stopListening = island.start();
    return () => void stopListening.then((stop) => stop());
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
</script>

<div class="fixed inset-0 flex items-end justify-center" class:invisible={island.hidden}>
  <div
    bind:this={surface}
    role="presentation"
    class="relative overflow-hidden bg-island text-island-text transition-[width,height,border-radius] duration-(--island-duration) ease-island {island.expanded
      ? 'h-[20rem] w-[28rem] rounded-island'
      : 'h-[2.5rem] w-[16rem] rounded-[1.25rem]'}"
    onwheel={cycleOnWheel}
    onpointerenter={() => island.cancelCollapse()}
    onpointerleave={() => island.scheduleCollapse()}
  >
    {#if island.expanded}
      <div class="absolute inset-0" in:fade={{ duration: 200, delay: 150 }} out:fade={{ duration: 100 }}>
        {#if island.focused}
          <Card activity={island.focused} />
        {:else}
          <IdleCard />
        {/if}
        <button
          class="absolute top-2 right-3 text-island-muted hover:text-island-text"
          type="button"
          aria-label="Collapse"
          onclick={() => island.setExpanded(false)}
        >
          ✕
        </button>
      </div>
    {:else}
      <button
        class="absolute inset-0 text-left"
        type="button"
        in:fade={{ duration: 200, delay: 150 }}
        out:fade={{ duration: 100 }}
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
