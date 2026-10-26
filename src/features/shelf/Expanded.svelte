<script lang="ts">
  import type { Activity } from "../../lib/contracts";
  import ShelfItem from "./ShelfItem.svelte";
  import type { ShelfView } from "./shelfView";

  let { activity }: { activity: Activity } = $props();

  const content = $derived(activity.expanded as ShelfView);
</script>

{#if content.view === "drop"}
  <div
    class="m-3 flex h-[calc(100%-1.5rem)] flex-col items-center justify-center gap-2 rounded-2xl border-2 border-dashed border-[#a78bfa] bg-[#a78bfa]/10 shadow-[inset_0_0_2.5rem_0_rgb(167_139_250/0.35)] transition-all duration-300"
  >
    <span class="animate-bounce text-3xl">📥</span>
    <p class="text-sm font-medium">Drop to keep files on the shelf</p>
  </div>
{:else}
  <div class="flex h-full flex-col gap-2 p-4">
    <p class="text-xs tracking-wide text-island-muted uppercase">
      Shelf · double-click to open, drag out to move
    </p>
    <ul class="flex min-h-0 flex-1 flex-col gap-1.5 overflow-y-auto">
      {#each content.files as entry (entry.path)}
        <ShelfItem {entry} activityId={activity.id} />
      {/each}
    </ul>
  </div>
{/if}
