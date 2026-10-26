<script lang="ts">
  import { useClock } from "../lib/clock.svelte";

  const clock = useClock();
</script>

<div class="flex h-full w-full items-center justify-center text-sm text-island-muted">
  {clock.time}
</div>
