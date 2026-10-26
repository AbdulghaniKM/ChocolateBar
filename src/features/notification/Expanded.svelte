<script lang="ts">
  import type { Activity } from "../../lib/contracts";

  let { activity }: { activity: Activity } = $props();

  const toast = $derived(activity.expanded as { title: string; body: string });
</script>

<div class="flex h-full flex-col gap-2 p-5">
  <div class="flex items-center gap-2 text-xs tracking-wide text-island-muted uppercase">
    <span>🔔</span>
    <span>Notification</span>
  </div>
  <p class="text-base font-semibold">{toast.title}</p>
  <p class="min-h-0 flex-1 overflow-auto text-sm leading-relaxed whitespace-pre-line text-island-text/90">
    {toast.body}
  </p>
</div>
