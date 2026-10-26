<script lang="ts">
  import type { Activity } from "../../lib/contracts";
  import type { KeyboardView } from "./types";

  let { activity }: { activity: Activity } = $props();

  const keyboard = $derived(activity.expanded as KeyboardView);
</script>

<div class="flex h-full flex-col items-center justify-center gap-5 px-6">
  <div
    class="flex size-[7rem] items-center justify-center rounded-[1.75rem] border border-white/10 bg-white/5 shadow-[0_0_2rem_rgb(255_255_255/0.04)] backdrop-blur"
  >
    {#key keyboard.layoutTag}
      <span class="tag text-[3rem] font-semibold tracking-[0.1em] text-island-text">{keyboard.layoutTag}</span>
    {/key}
  </div>
  <div class="flex flex-col items-center gap-2">
    <span class="text-[1rem] text-island-text">{keyboard.layoutName}</span>
    <span
      class="rounded-pill border px-3 py-1 text-[0.75rem] font-semibold tracking-wider transition-colors duration-300
        {keyboard.capsLock
        ? 'border-accent/60 bg-accent/15 text-accent'
        : 'border-white/10 bg-white/5 text-island-muted'}"
    >
      {keyboard.capsLock ? "CAPS LOCK ON" : "Caps Lock off"}
    </span>
  </div>
</div>

<style>
  .tag {
    animation: tag-in 380ms cubic-bezier(0.32, 0.72, 0, 1);
  }
  @keyframes tag-in {
    from {
      opacity: 0;
      transform: scale(0.8);
    }
  }
</style>
