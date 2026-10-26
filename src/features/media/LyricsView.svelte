<script lang="ts">
  import { fly } from "svelte/transition";
  import { currentLyricIndex } from "./format";
  import type { LyricLine } from "./types";

  let { lines, positionMs }: { lines: LyricLine[]; positionMs: number } = $props();

  const index = $derived(currentLyricIndex(lines, positionMs));
  const previous = $derived(index > 0 ? lines[index - 1].text : "");
  const current = $derived(index >= 0 ? lines[index].text : "");
  const next = $derived(index + 1 < lines.length ? lines[index + 1].text : "");
</script>

<div class="flex h-[3.75rem] flex-col items-center justify-center gap-0.5 overflow-hidden text-center">
  {#key index}
    <div class="flex w-full flex-col items-center gap-0.5" in:fly={{ y: 10, duration: 260 }}>
      <p class="w-full truncate text-[0.6875rem] text-island-muted/60">{previous}</p>
      <p class="w-full truncate text-sm font-semibold text-island-text">{current || "..."}</p>
      <p class="w-full truncate text-[0.6875rem] text-island-muted/80">{next}</p>
    </div>
  {/key}
</div>
