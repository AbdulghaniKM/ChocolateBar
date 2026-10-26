<script lang="ts">
  import type { Snippet } from "svelte";

  const RADIUS = 44;
  const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

  let {
    fraction,
    accent,
    strokeWidth = 7,
    children,
  }: { fraction: number; accent: string; strokeWidth?: number; children?: Snippet } = $props();

  const dashOffset = $derived(CIRCUMFERENCE * (1 - Math.min(1, Math.max(0, fraction))));
</script>

<div class="relative size-full">
  <svg class="size-full -rotate-90" viewBox="0 0 100 100" role="presentation">
    <circle
      cx="50"
      cy="50"
      r={RADIUS}
      fill="none"
      stroke="var(--color-island-raised)"
      stroke-width={strokeWidth}
    />
    <circle
      class="transition-[stroke-dashoffset] duration-200 ease-linear"
      cx="50"
      cy="50"
      r={RADIUS}
      fill="none"
      stroke={accent}
      stroke-width={strokeWidth}
      stroke-linecap="round"
      stroke-dasharray={CIRCUMFERENCE}
      stroke-dashoffset={dashOffset}
    />
  </svg>
  {#if children}
    <div class="absolute inset-0 flex flex-col items-center justify-center">
      {@render children()}
    </div>
  {/if}
</div>
