<script lang="ts">
  import { useStream } from "../../lib/stream.svelte";

  let { active = true }: { active?: boolean } = $props();

  const BAR_COUNT = 12;
  const MIN_HEIGHT_PERCENT = 12;
  const spectrum = useStream<number[]>("visualizer", []);

  const bars = $derived(
    Array.from({ length: BAR_COUNT }, (_, index) => {
      const level = active ? (spectrum.value[index] ?? 0) : 0;
      return MIN_HEIGHT_PERCENT + level * (100 - MIN_HEIGHT_PERCENT);
    }),
  );
</script>

<div class="flex h-full items-end gap-[0.125rem]" aria-hidden="true">
  {#each bars as height, index (index)}
    <span
      class="w-[0.1875rem] rounded-pill bg-accent transition-[height] duration-100 ease-out"
      style:height="{height}%"
      style:opacity={0.55 + height / 230}
    ></span>
  {/each}
</div>
