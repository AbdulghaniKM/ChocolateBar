<script lang="ts">
  import { useClock } from "../lib/clock.svelte";

  const clock = useClock();
</script>

<div class="flex h-full w-full items-center justify-between px-4">
  <span class="flex items-center gap-2">
    <span class="size-1.5 rounded-full bg-accent shadow-[0_0_0.5rem_var(--color-accent)]"></span>
    <span class="text-[0.75rem] font-medium text-island-muted">{clock.shortDate}</span>
  </span>
  <span class="text-[0.8125rem] font-semibold text-island-text tabular-nums">{clock.time}</span>
</div>
