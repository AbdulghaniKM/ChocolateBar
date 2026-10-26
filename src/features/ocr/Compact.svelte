<script lang="ts">
  import Glyph from "../../shell/Glyph.svelte";
  import type { Activity } from "../../lib/contracts";

  let { activity }: { activity: Activity } = $props();
</script>

<div class="flex min-w-0 flex-1 items-center gap-2 px-3 text-sm">
  <Glyph glyph={activity.compact.icon} class="size-4 text-accent" />
  <span class="min-w-0 flex-1 truncate font-medium">{activity.compact.title}</span>
  {#if activity.compact.subtitle}
    <span class="shrink-0 rounded-pill bg-island-raised px-2 text-xs text-island-muted">
      {activity.compact.subtitle}
    </span>
  {/if}
</div>
