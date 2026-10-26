<script lang="ts">
  import type { Activity } from "../../lib/contracts";

  type OcrView =
    | { view: "text"; text: string; copied: boolean; characterCount: number }
    | { view: "message"; message: string };

  let { activity }: { activity: Activity } = $props();

  const content = $derived(activity.expanded as OcrView);
</script>

<div class="flex h-full flex-col gap-3 p-4">
  <div class="flex items-center gap-2 text-xs tracking-wide text-island-muted uppercase">
    <span>📝</span>
    <span>Recognized text</span>
    {#if content.view === "text" && content.copied}
      <span class="ml-auto rounded-pill bg-success/20 px-2 py-0.5 text-success normal-case">Copied</span>
    {/if}
  </div>
  {#if content.view === "text"}
    <p
      class="min-h-0 flex-1 overflow-auto rounded-xl border border-island-border bg-island-raised p-3 text-sm leading-relaxed whitespace-pre-wrap select-text"
      dir="auto"
    >
      {content.text}
    </p>
  {:else}
    <p class="flex flex-1 items-center justify-center text-sm text-island-muted">{content.message}</p>
  {/if}
</div>
