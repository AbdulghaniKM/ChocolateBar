<script lang="ts">
  import KeyRound from "@lucide/svelte/icons/key-round";
  import LoaderCircle from "@lucide/svelte/icons/loader-circle";
  import type { Activity } from "../../lib/contracts";
  import type { ClaudeView } from "./claudeView";
  import ConversationView from "./ConversationView.svelte";
  import ReplyBox from "./ReplyBox.svelte";
  import SessionChips from "./SessionChips.svelte";

  let { activity }: { activity: Activity } = $props();

  const view = $derived(activity.expanded as ClaudeView);
  const working = $derived(view.view === "session" && (view.state === "thinking" || view.state === "tool" || view.state === "started"));
</script>

{#if view.view === "permission"}
  <div class="flex h-full flex-col gap-3 px-4 pb-3">
    <div class="flex items-center gap-2.5">
      <span class="grid size-9 place-items-center rounded-xl bg-accent/15 text-accent"><KeyRound class="size-4" /></span>
      <div class="min-w-0">
        <p class="text-sm font-semibold">Claude wants to use {view.toolName}</p>
        {#if view.project}
          <p class="truncate text-xs text-island-muted">{view.project}</p>
        {/if}
      </div>
      <div class="ml-auto flex shrink-0 gap-1"><SessionChips {activity} /></div>
    </div>
    <pre
      class="island-scroll min-h-0 flex-1 overflow-auto rounded-xl border border-island-border bg-island-raised p-3 font-mono text-xs leading-relaxed break-all whitespace-pre-wrap text-island-text">{view.inputPreview}</pre>
  </div>
{:else}
  <div class="flex h-full flex-col px-4 pb-3">
    <div class="flex items-center gap-2 pb-2">
      <p class="min-w-0 truncate text-[0.9375rem] font-semibold">{view.project ?? "Claude Code"}</p>
      <div class="ml-auto flex shrink-0 gap-1"><SessionChips {activity} /></div>
    </div>
    <div class="min-h-0 flex-1 border-t border-island-border/60 pt-2">
      {#if view.sessionId}
        <ConversationView sessionId={view.sessionId} />
      {/if}
    </div>
    {#if view.sessionId}
      <div class="mt-2 shrink-0"><ReplyBox sessionId={view.sessionId} /></div>
    {/if}
    <div
      class="mt-2 flex h-8 shrink-0 items-center gap-2 rounded-xl border px-3 text-[0.75rem]
        {view.state === 'waiting' ? 'border-accent/50 bg-accent/10' : 'border-island-border bg-island-raised/70'}"
      aria-live="polite"
    >
      {#if working}
        <LoaderCircle class="size-3.5 shrink-0 animate-spin text-accent" />
      {:else}
        <span class="size-2 shrink-0 rounded-full {view.state === 'waiting' ? 'animate-pulse bg-accent' : view.state === 'done' ? 'bg-success' : 'bg-island-subtle'}"></span>
      {/if}
      <span class="min-w-0 truncate {working ? 'text-island-text' : 'text-island-muted'}">{view.statusLine}</span>
    </div>
  </div>
{/if}
