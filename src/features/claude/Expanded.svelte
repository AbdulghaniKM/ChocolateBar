<script lang="ts">
  import type { Activity } from "../../lib/contracts";
  import type { ClaudeView } from "./claudeView";
  import SessionChips from "./SessionChips.svelte";

  let { activity }: { activity: Activity } = $props();

  const view = $derived(activity.expanded as ClaudeView);
</script>

{#if view.view === "permission"}
  <div class="flex h-full flex-col gap-3 p-4">
    <div class="flex items-center gap-2">
      <span class="grid size-8 place-items-center rounded-xl bg-accent/15 text-base">🔐</span>
      <div class="min-w-0">
        <p class="text-sm font-semibold">Claude wants to use {view.toolName}</p>
        {#if view.project}
          <p class="truncate text-xs text-island-muted">{view.project}</p>
        {/if}
      </div>
      <div class="ml-auto flex shrink-0 gap-1"><SessionChips {activity} /></div>
    </div>
    <pre
      class="min-h-0 flex-1 overflow-auto rounded-xl border border-island-border bg-island-raised p-3 font-mono text-xs leading-relaxed whitespace-pre-wrap break-all text-island-text">{view.inputPreview}</pre>
  </div>
{:else}
  <div class="flex h-full flex-col justify-center gap-3 p-5">
    <div class="flex items-center gap-3">
      <span
        class="grid size-10 place-items-center rounded-2xl text-lg transition-colors duration-300"
        style:background-color="color-mix(in srgb, {activity.compact.accent} 18%, transparent)"
        style:color={activity.compact.accent}
      >
        ✦
      </span>
      <div class="min-w-0">
        <p class="truncate text-base font-semibold">{view.project ?? "Claude Code"}</p>
        <p class="text-xs tracking-wide text-island-muted uppercase">{view.state}</p>
      </div>
      <div class="ml-auto flex shrink-0 gap-1"><SessionChips {activity} /></div>
    </div>
    <div class="flex items-center gap-2 rounded-xl border border-island-border bg-island-raised px-3 py-2 text-sm">
      {#if view.state === "thinking" || view.state === "tool"}
        <span class="size-2 shrink-0 animate-pulse rounded-full bg-accent"></span>
      {/if}
      <span class="min-w-0 truncate">{view.statusLine}</span>
    </div>
  </div>
{/if}
