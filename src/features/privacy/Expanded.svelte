<script lang="ts">
  import Glyph from "../../shell/Glyph.svelte";
  import type { Activity } from "../../lib/contracts";
  import type { PrivacyExpanded } from "./types";

  let { activity }: { activity: Activity } = $props();

  const privacy = $derived(activity.expanded as PrivacyExpanded);
</script>

<div class="flex h-full flex-col gap-4 px-5 pb-4">
  <div class="flex items-center gap-3">
    <span
      class="flex size-[3rem] items-center justify-center rounded-2xl ring-1 ring-white/10"
      style:color={activity.compact.accent}
      style:background-color="color-mix(in srgb, {activity.compact.accent ?? 'transparent'} 22%, transparent)"
    >
      <Glyph glyph={activity.compact.icon} class="size-6" />
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
        <li class="flex items-center gap-2 tile rounded-xl px-3 py-2 text-sm text-island-text">
          <span class="size-[0.375rem] rounded-full" style:background-color={activity.compact.accent}></span>
          <span class="truncate">{app}</span>
        </li>
      {/each}
    </ul>
  {/if}
</div>
