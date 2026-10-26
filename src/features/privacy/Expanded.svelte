<script lang="ts">
  import type { Activity } from "../../lib/contracts";
  import type { PrivacyExpanded } from "./types";

  let { activity }: { activity: Activity } = $props();

  const privacy = $derived(activity.expanded as PrivacyExpanded);
</script>

<div class="flex h-full flex-col gap-4 p-5">
  <div class="flex items-center gap-3">
    <span
      class="flex size-[3rem] items-center justify-center rounded-2xl text-2xl ring-1 ring-white/10"
      style:background-color="color-mix(in srgb, {activity.compact.accent ?? 'transparent'} 22%, transparent)"
    >
      {activity.compact.icon}
    </span>
    <div class="flex min-w-0 flex-col">
      <p class="truncate text-base font-semibold text-island-text">{activity.compact.title}</p>
      {#if privacy.kind === "mic"}
        <p class="text-xs text-island-muted">{privacy.muted ? "Apps cannot hear you" : "Apps can hear you"}</p>
      {/if}
    </div>
  </div>
  {#if privacy.apps.length > 0}
    <ul class="flex flex-col gap-1.5 overflow-y-auto">
      {#each privacy.apps as app (app)}
        <li class="flex items-center gap-2 rounded-xl bg-white/5 px-3 py-2 text-sm text-island-text">
          <span class="size-[0.375rem] rounded-full" style:background-color={activity.compact.accent}></span>
          <span class="truncate">{app}</span>
        </li>
      {/each}
    </ul>
  {/if}
</div>
