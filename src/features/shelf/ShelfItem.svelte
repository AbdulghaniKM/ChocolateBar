<script lang="ts">
  import X from "@lucide/svelte/icons/x";
  import File from "@lucide/svelte/icons/file";
  import Folder from "@lucide/svelte/icons/folder";
  import { startDrag } from "@crabnebula/tauri-plugin-drag";
  import { activityAction, sourceCommand } from "../../lib/ipc";
  import type { ShelfEntry } from "./shelfView";

  let { entry, activityId }: { entry: ShelfEntry; activityId: string } = $props();

  const DRAG_THRESHOLD = 6;
  let pressedAt: { x: number; y: number } | null = null;

  const rememberPress = (event: PointerEvent) => (pressedAt = { x: event.clientX, y: event.clientY });

  async function dragOutAfterMove(event: PointerEvent) {
    if (!pressedAt || event.buttons !== 1) return;
    const moved = Math.hypot(event.clientX - pressedAt.x, event.clientY - pressedAt.y);
    if (moved < DRAG_THRESHOLD) return;
    pressedAt = null;
    await dragOut();
  }

  async function dragOut() {
    try {
      const { path: icon } = await sourceCommand<{ path: string }>("shelf", "drag_icon");
      await startDrag({ item: [entry.path], icon });
    } catch (error) {
      console.warn("shelf drag-out failed", error);
    }
  }
</script>

<li
  class="group flex items-center gap-2 rounded-xl bg-island-raised px-3 py-1.5 text-sm transition-colors hover:bg-island-hover"
>
  <button
    class="flex min-w-0 flex-1 cursor-grab items-center gap-2 text-left active:cursor-grabbing"
    type="button"
    title={entry.path}
    onpointerdown={rememberPress}
    onpointermove={dragOutAfterMove}
    ondblclick={() => void activityAction(activityId, "open", entry.path)}
  >
    {#if entry.isDirectory}<Folder class="size-4 shrink-0 text-accent" />{:else}<File class="size-4 shrink-0 text-island-muted" />{/if}
    <span class="truncate">{entry.name}</span>
  </button>
  <button
    class="grid size-7 place-items-center rounded-full text-island-muted opacity-0 transition-opacity group-hover:opacity-100 group-focus-within:opacity-100 hover:text-danger focus-visible:opacity-100"
    type="button"
    aria-label="Remove {entry.name}"
    onclick={() => void activityAction(activityId, "remove", entry.path)}
  >
    <X class="size-4" />
  </button>
</li>
