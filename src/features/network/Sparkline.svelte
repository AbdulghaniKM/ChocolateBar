<script lang="ts">
  let { samples, color }: { samples: number[]; color: string } = $props();

  const WIDTH = 100;
  const HEIGHT = 30;

  const points = $derived(buildPoints(samples));

  function buildPoints(values: number[]) {
    if (values.length < 2) return "";
    const peak = Math.max(...values, 1);
    const step = WIDTH / (values.length - 1);
    return values.map((value, index) => `${(index * step).toFixed(2)},${(HEIGHT - (value / peak) * HEIGHT).toFixed(2)}`).join(" ");
  }
</script>

<svg class="h-full w-full overflow-visible" viewBox="0 0 {WIDTH} {HEIGHT}" preserveAspectRatio="none" aria-hidden="true">
  {#if points}
    <polyline
      {points}
      fill="none"
      stroke={color}
      stroke-width="1.6"
      stroke-linecap="round"
      stroke-linejoin="round"
      vector-effect="non-scaling-stroke"
    />
  {/if}
</svg>
