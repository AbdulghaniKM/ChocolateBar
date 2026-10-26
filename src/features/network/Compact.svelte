<script lang="ts">
  import type { Activity } from "../../lib/contracts";
  import { isEventView, type NetworkExpanded } from "./types";

  let { activity }: { activity: Activity } = $props();

  const view = $derived(activity.expanded as NetworkExpanded);
  const status = $derived(isEventView(view) ? null : view);
</script>

<div class="flex h-full w-full min-w-0 items-center gap-2.5 px-4">
  <span class="text-[1rem]" style:color={activity.compact.accent}>{activity.compact.icon}</span>
  <span class="min-w-0 truncate text-[0.8125rem] font-medium text-island-text">{activity.compact.title}</span>
  {#if status?.vpn}
    <span class="rounded-pill bg-violet-400/20 px-1.5 py-px text-[0.5625rem] font-semibold tracking-wider text-violet-300">
      VPN
    </span>
  {/if}
  {#if status && status.showSpeed && status.link !== "offline"}
    <span class="ml-auto shrink-0 text-[0.6875rem] text-island-muted tabular-nums">
      <span class="text-sky-300">↓</span> {status.downloadText}
      <span class="ml-1 text-emerald-300">↑</span> {status.uploadText}
    </span>
  {/if}
</div>
