<script lang="ts">
  import type { WindowLayout } from "../../lib/contracts";

  let { layout, class: className = "h-3.5 w-5" }: { layout: WindowLayout; class?: string } = $props();

  const AREAS: Record<WindowLayout, [number, number, number, number] | null> = {
    none: null,
    maximize: [1.5, 1.5, 17, 11],
    left: [1.5, 1.5, 8, 11],
    right: [10.5, 1.5, 8, 11],
    "top-left": [1.5, 1.5, 8, 5],
    "top-right": [10.5, 1.5, 8, 5],
    "bottom-left": [1.5, 7.5, 8, 5],
    "bottom-right": [10.5, 7.5, 8, 5],
  };
  const area = $derived(AREAS[layout]);
</script>

<svg class={className} viewBox="0 0 20 14" aria-hidden="true">
  <rect x="0.5" y="0.5" width="19" height="13" rx="2.5" fill="none" stroke="currentColor" stroke-opacity="0.45" />
  {#if area}
    <rect x={area[0]} y={area[1]} width={area[2]} height={area[3]} rx="1.2" fill="currentColor" />
  {:else}
    <path d="M7 7h6" stroke="currentColor" stroke-opacity="0.6" stroke-linecap="round" />
  {/if}
</svg>
