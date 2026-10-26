<script lang="ts">
  import { currentLyricIndex } from "./format";
  import type { LyricLine } from "./types";

  let { lines, positionMs }: { lines: LyricLine[]; positionMs: number } = $props();

  let viewport = $state<HTMLDivElement>();
  let list = $state<HTMLDivElement>();
  let layoutVersion = $state(0);
  let offset = $state(0);

  const index = $derived(currentLyricIndex(lines, positionMs));

  $effect(() => {
    if (!viewport || !list) return;
    const observer = new ResizeObserver(() => (layoutVersion += 1));
    observer.observe(viewport);
    observer.observe(list);
    return () => observer.disconnect();
  });

  $effect(() => {
    void layoutVersion;
    const current = list?.children[Math.max(0, index)] as HTMLElement | undefined;
    if (!viewport || !current) return;
    offset = current.offsetTop + current.offsetHeight / 2 - viewport.clientHeight / 2;
  });
</script>

<div
  bind:this={viewport}
  class="relative h-full overflow-hidden [mask-image:linear-gradient(transparent,black_25%,black_75%,transparent)]"
  aria-live="polite"
>
  <div
    bind:this={list}
    class="relative flex flex-col gap-1.5 px-1 transition-transform duration-500 ease-out-soft"
    style:transform="translateY({-offset}px)"
  >
    {#each lines as line, lineIndex (lineIndex)}
      <p
        class="text-center leading-snug text-balance transition-[color,opacity] duration-300
          {lineIndex === index
          ? 'text-[0.9375rem] font-semibold text-island-text'
          : 'text-[0.8125rem] text-island-muted opacity-60'}"
        dir="auto"
      >
        {line.text || "♪"}
      </p>
    {/each}
  </div>
</div>
