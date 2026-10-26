<script lang="ts">
  import type { Activity } from "../../lib/contracts";

  let { activity }: { activity: Activity } = $props();
</script>

<div class="flex min-w-0 items-center gap-2 px-3 text-sm">
  <span>{activity.compact.icon}</span>
  <span class="truncate">{activity.compact.title}</span>
</div>
