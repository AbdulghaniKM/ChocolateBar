<script lang="ts">
  import { activityAction } from "../../lib/ipc";
  import { formatClock } from "./format";

  let {
    activityId,
    positionMs,
    durationMs,
  }: { activityId: string; positionMs: number; durationMs: number } = $props();

  const fraction = $derived(durationMs > 0 ? Math.min(1, positionMs / durationMs) : 0);

  function seek(event: MouseEvent) {
    if (durationMs <= 0) return;
    const track = event.currentTarget as HTMLElement;
    const bounds = track.getBoundingClientRect();
    const target = ((event.clientX - bounds.left) / bounds.width) * durationMs;
    void activityAction(activityId, "seek", Math.round(Math.max(0, Math.min(durationMs, target))));
  }
</script>

<div class="flex items-center gap-2 text-[0.6875rem] tabular-nums text-island-muted">
  <span class="w-[2.25rem] text-right">{formatClock(positionMs)}</span>
  <button
    class="group relative h-[1rem] flex-1 cursor-pointer"
    type="button"
    aria-label="Seek"
    disabled={durationMs <= 0}
    onclick={seek}
  >
    <span class="absolute inset-x-0 top-1/2 h-[0.25rem] -translate-y-1/2 overflow-hidden rounded-pill bg-white/10 transition-[height] group-hover:h-[0.375rem]">
      <span class="block h-full rounded-pill bg-island-text" style:width="{fraction * 100}%"></span>
    </span>
  </button>
  <span class="w-[2.25rem]">{durationMs > 0 ? formatClock(durationMs) : "--:--"}</span>
</div>
