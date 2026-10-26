<script lang="ts">
  import { useClock } from "../lib/clock.svelte";
  import { registry } from "../features/registry";

  const clock = useClock();
  const launchers = Object.values(registry).flatMap((entry) => (entry.Launcher ? [entry.Launcher] : []));
</script>

<div class="flex h-full w-full flex-col items-center justify-center gap-3">
  <p class="text-5xl font-light tabular-nums">{clock.time}</p>
  <p class="text-sm text-island-muted">{clock.date}</p>
  <div class="flex items-center gap-2">
    {#each launchers as Launcher}
      <Launcher />
    {/each}
  </div>
</div>
