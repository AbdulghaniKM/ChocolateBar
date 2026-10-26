<script lang="ts">
  import Zap from "@lucide/svelte/icons/zap";
  import { LOW_BATTERY_PERCENT } from "./types";

  let {
    label,
    percent,
    charging,
    dimmed = false,
  }: { label: string; percent: number | null; charging: boolean; dimmed?: boolean } = $props();

  const isLow = $derived(percent !== null && percent <= LOW_BATTERY_PERCENT);
  const ringColor = $derived(isLow ? "var(--color-danger)" : charging ? "var(--color-success)" : "var(--color-accent)");
</script>

<div class="flex flex-col items-center gap-1.5 transition-opacity duration-300 {dimmed ? 'opacity-40' : ''}">
  <div class="relative size-[4.5rem]">
    <svg class="size-full -rotate-90" viewBox="0 0 36 36" aria-hidden="true">
      <circle cx="18" cy="18" r="15.9155" fill="none" stroke="var(--color-island-raised)" stroke-width="3" />
      <circle
        class="ring"
        class:charging
        cx="18"
        cy="18"
        r="15.9155"
        fill="none"
        stroke={ringColor}
        stroke-width="3"
        stroke-linecap="round"
        stroke-dasharray="{percent ?? 0} 100"
      />
    </svg>
    <div class="absolute inset-0 flex flex-col items-center justify-center leading-none">
      <span class="text-[1.0625rem] font-semibold tabular-nums text-island-text">
        {percent === null ? "-" : percent}<span class="text-[0.625rem] text-island-muted">{percent === null ? "" : "%"}</span>
      </span>
      {#if charging}
        <Zap class="mt-0.5 size-2.5 text-success" fill="currentColor" strokeWidth={0} />
      {/if}
    </div>
  </div>
  <span class="text-[0.6875rem] tracking-wider text-island-muted uppercase">{label}</span>
</div>

<style>
  .ring {
    transition:
      stroke-dasharray 600ms cubic-bezier(0.32, 0.72, 0, 1),
      stroke 300ms;
  }
  .ring.charging {
    animation: ring-pulse 1.6s ease-in-out infinite;
  }
  @keyframes ring-pulse {
    50% {
      opacity: 0.55;
    }
  }
</style>
