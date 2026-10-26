<script lang="ts">
  import Inbox from "@lucide/svelte/icons/inbox";
  import type { Activity } from "../../lib/contracts";
  import ShelfItem from "./ShelfItem.svelte";
  import type { ShelfView } from "./shelfView";

  let { activity }: { activity: Activity } = $props();

  const content = $derived(activity.expanded as ShelfView);
</script>

{#if content.view === "drop"}
  <div
    class="m-3 flex h-[calc(100%-1.5rem)] flex-col items-center justify-center gap-2 rounded-2xl border-2 border-dashed border-berry bg-berry/10 shadow-[inset_0_0_2.5rem_0_rgb(199_161_255/0.3)] transition-all duration-300"
  >
    <Inbox class="size-8 animate-bounce text-berry" />
    <p class="text-sm font-medium">Drop to keep files on the shelf</p>
  </div>
{:else}
  <div class="flex h-full flex-col gap-2 px-4 pb-3">
    <p class="text-[0.75rem] text-island-subtle">Double-click to open. Drag out to move.</p>
    <ul class="island-scroll flex min-h-0 flex-1 flex-col gap-1.5 overflow-y-auto">
      {#each content.files as entry (entry.path)}
        <ShelfItem {entry} activityId={activity.id} />
      {/each}
    </ul>
  </div>
{/if}
