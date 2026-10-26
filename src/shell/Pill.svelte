<script lang="ts">
  import type { Activity } from "../lib/contracts";
  import { registry } from "../features/registry";

  let { activity }: { activity: Activity } = $props();

  const Compact = $derived(registry[activity.kind].Compact);
  const progressPercent = $derived(Math.round((activity.compact.progress ?? 0) * 100));
</script>

<div class="relative flex h-full w-full items-center">
  {#if Compact}
    <Compact {activity} />
  {:else}
    <div class="flex min-w-0 items-center gap-2 px-3 text-sm">
      <span>{activity.compact.icon}</span>
      <span class="truncate">{activity.compact.title}</span>
      {#if activity.compact.subtitle}
        <span class="truncate text-island-muted">{activity.compact.subtitle}</span>
      {/if}
      {#if activity.compact.badge}
        <span class="ml-auto rounded-pill bg-island-raised px-2 text-xs">{activity.compact.badge}</span>
      {/if}
    </div>
  {/if}
  {#if activity.compact.progress !== null}
    <div class="absolute inset-x-4 bottom-0 h-[0.1875rem] rounded-pill bg-island-raised">
      <div
        class="h-full rounded-pill bg-accent transition-[width] duration-300"
        style:width="{progressPercent}%"
        style:background-color={activity.compact.accent}
      ></div>
    </div>
  {/if}
</div>
