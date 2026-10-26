<script lang="ts">
  import type { Activity } from "../../lib/contracts";
  import type { KeyboardView } from "./types";

  let { activity }: { activity: Activity } = $props();

  const keyboard = $derived(activity.expanded as KeyboardView);
</script>

<div class="flex h-full w-full min-w-0 items-center justify-center gap-3 px-4">
  <span class="text-[1rem] text-island-muted">{activity.compact.icon}</span>
  {#key keyboard.layoutTag}
    <span class="tag text-[1.125rem] font-semibold tracking-[0.12em] text-island-text">
      {keyboard.layoutTag}
    </span>
  {/key}
  <span
    class="rounded-pill px-2 py-0.5 text-[0.625rem] font-semibold tracking-wider transition-all duration-300 ease-island
      {keyboard.capsLock
      ? 'scale-100 bg-accent text-island opacity-100'
      : 'scale-75 bg-island-raised text-island-muted opacity-40'}"
  >
    CAPS
  </span>
</div>

<style>
  .tag {
    animation: tag-in 320ms cubic-bezier(0.32, 0.72, 0, 1);
  }
  @keyframes tag-in {
    from {
      opacity: 0;
      transform: translateY(0.4rem) scale(0.9);
    }
  }
</style>
