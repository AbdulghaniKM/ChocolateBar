<script lang="ts">
  import type { Activity } from "../../lib/contracts";

  let { activity }: { activity: Activity } = $props();
</script>

<div class="flex min-w-0 flex-1 items-center gap-2 px-3 text-sm">
  <span class="relative flex size-2 shrink-0">
    <span
      class="absolute inline-flex size-full rounded-full opacity-60 {activity.alert ? 'animate-ping' : ''}"
      style:background-color={activity.compact.accent}
    ></span>
    <span class="relative inline-flex size-2 rounded-full" style:background-color={activity.compact.accent}></span>
  </span>
  <span class="max-w-[5.5rem] shrink-0 truncate font-medium">{activity.compact.title}</span>
  {#if activity.compact.subtitle}
    <span class="min-w-0 flex-1 truncate text-island-muted">{activity.compact.subtitle}</span>
  {/if}
</div>
