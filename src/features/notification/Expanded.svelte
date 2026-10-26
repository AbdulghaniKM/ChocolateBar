<script lang="ts">
  import type { Activity } from "../../lib/contracts";

  let { activity }: { activity: Activity } = $props();
</script>

<div class="flex flex-col gap-1 p-4">
  <p class="text-base font-medium">{activity.compact.title}</p>
  {#if activity.compact.subtitle}
    <p class="text-sm text-island-muted">{activity.compact.subtitle}</p>
  {/if}
</div>
