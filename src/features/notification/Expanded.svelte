<script lang="ts">
  import type { Activity } from "../../lib/contracts";

  let { activity }: { activity: Activity } = $props();

  const toast = $derived(activity.expanded as { title: string; body: string });
</script>

<div class="flex h-full flex-col gap-2 px-5 pb-4">
  <p class="text-base font-semibold">{toast.title}</p>
  <p class="min-h-0 flex-1 overflow-auto text-sm leading-relaxed whitespace-pre-line text-island-text/90">
    {toast.body}
  </p>
</div>
