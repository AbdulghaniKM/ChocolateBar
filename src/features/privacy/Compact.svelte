<script lang="ts">
  import type { Activity } from "../../lib/contracts";
  import type { PrivacyExpanded } from "./types";

  let { activity }: { activity: Activity } = $props();

  const privacy = $derived(activity.expanded as PrivacyExpanded);
  const isLive = $derived(privacy.kind !== "mute" && !(privacy.kind === "mic" && privacy.muted));
</script>

<div class="flex h-full w-full min-w-0 items-center gap-2.5 px-3.5">
  <span class="relative flex size-[0.625rem] shrink-0">
    {#if isLive}
      <span
        class="absolute inline-flex size-full animate-ping rounded-full opacity-60"
        style:background-color={activity.compact.accent}
      ></span>
    {/if}
    <span class="relative inline-flex size-full rounded-full" style:background-color={activity.compact.accent}></span>
  </span>
  <span class="shrink-0 text-base">{activity.compact.icon}</span>
  <div class="flex min-w-0 flex-1 flex-col leading-tight">
    <span class="truncate text-[0.8125rem] font-medium text-island-text">{activity.compact.title}</span>
    {#if activity.compact.subtitle}
      <span class="truncate text-[0.6875rem] text-island-muted">{activity.compact.subtitle}</span>
    {/if}
  </div>
</div>
