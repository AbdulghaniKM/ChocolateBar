<script lang="ts">
  import { NOISE_OPTIONS, type NoiseMode } from "./types";

  let { current, onselect }: { current: NoiseMode | null; onselect: (mode: NoiseMode) => void } = $props();

  const activeIndex = $derived(NOISE_OPTIONS.findIndex((option) => option.mode === current));
</script>

<div
  class="relative flex w-full tile rounded-pill p-[0.1875rem]"
  role="radiogroup"
  aria-label="Noise control"
>
  {#if activeIndex >= 0}
    <span
      class="absolute inset-y-[0.1875rem] left-[0.1875rem] rounded-pill bg-accent shadow-[0_0_0.75rem_rgb(234_166_96/0.3)] transition-transform duration-300 ease-island"
      style:width="calc((100% - 0.375rem) / {NOISE_OPTIONS.length})"
      style:transform="translateX({activeIndex * 100}%)"
    ></span>
  {/if}
  {#each NOISE_OPTIONS as option (option.mode)}
    <button
      class="relative z-10 flex-1 rounded-pill py-1.5 text-[0.75rem] font-medium transition-colors duration-200
        {option.mode === current ? 'text-on-accent' : 'text-island-muted hover:text-island-text'}"
      type="button"
      role="radio"
      aria-checked={option.mode === current}
      onclick={() => onselect(option.mode)}
    >
      {option.label}
    </button>
  {/each}
</div>
