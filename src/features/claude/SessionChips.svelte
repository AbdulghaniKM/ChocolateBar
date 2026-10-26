<script lang="ts">
  import type { Activity } from "../../lib/contracts";
  import { modelLabel, permissionModeLabel, type ClaudeView } from "./claudeView";

  let { activity }: { activity: Activity } = $props();

  const view = $derived(activity.expanded as ClaudeView);
  const permissionMode = $derived(view.permissionMode);
  const model = $derived(view.view === "session" ? view.model : null);
</script>

{#if permissionMode}
  <span class="shrink-0 rounded-pill bg-island-raised px-2 py-0.5 text-[0.65rem] text-island-muted">
    {permissionModeLabel(permissionMode)}
  </span>
{/if}
{#if model}
  <span class="max-w-[6rem] shrink-0 truncate rounded-pill bg-island-raised px-2 py-0.5 text-[0.65rem] text-island-muted">
    {modelLabel(model)}
  </span>
{/if}
