<script lang="ts">
  import type { Activity } from "../lib/contracts";
  import { registry } from "../features/registry";
  import ActionBar from "./ActionBar.svelte";

  let { activity }: { activity: Activity } = $props();

  const Expanded = $derived(registry[activity.kind].Expanded);
</script>

<div class="flex h-full w-full flex-col">
  <div class="min-h-0 flex-1 overflow-hidden">
    <Expanded {activity} />
  </div>
  <ActionBar activityId={activity.id} actions={activity.actions} />
</div>
