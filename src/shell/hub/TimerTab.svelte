<script lang="ts">
  import { useClock } from "../../lib/clock.svelte";
  import { registry } from "../../features/registry";

  const clock = useClock();
  const launchers = Object.values(registry).flatMap((entry) => (entry.Launcher ? [entry.Launcher] : []));
</script>

<div class="flex h-full flex-col items-center justify-center gap-5">
  <div class="flex flex-col items-center gap-1">
    <p class="font-display text-[3.25rem] leading-none font-semibold tracking-tight tabular-nums">{clock.time}</p>
    <p class="text-[0.8125rem] text-island-muted">{clock.date}</p>
  </div>
  <div class="flex flex-wrap items-center justify-center gap-2">
    {#each launchers as Launcher}
      <Launcher />
    {/each}
  </div>
</div>
