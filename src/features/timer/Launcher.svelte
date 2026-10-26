<script lang="ts">
  import { sourceCommand } from "../../lib/ipc";

  const COUNTDOWN_PRESET_MINUTES = [1, 5, 10, 25];
  const SECONDS_PER_MINUTE = 60;

  let choosingDuration = $state(false);

  async function start(timer: "countdown" | "stopwatch" | "pomodoro", seconds?: number) {
    try {
      await sourceCommand("timer", "start", { timer, seconds });
    } catch (error) {
      console.warn("timer start failed", error);
    }
    choosingDuration = false;
  }

  const buttonClasses =
    "flex h-8 min-w-8 items-center justify-center rounded-full bg-island-raised px-2 text-sm transition-[transform,background-color] duration-200 ease-island hover:scale-105 hover:bg-island-border active:scale-95";
</script>

<div class="flex items-center gap-2">
  {#if choosingDuration}
    {#each COUNTDOWN_PRESET_MINUTES as minutes (minutes)}
      <button
        class={buttonClasses}
        type="button"
        onclick={() => start("countdown", minutes * SECONDS_PER_MINUTE)}
      >
        {minutes}m
      </button>
    {/each}
    <button class={buttonClasses} type="button" aria-label="Back" onclick={() => (choosingDuration = false)}>
      &larr;
    </button>
  {:else}
    <button class={buttonClasses} type="button" title="Pomodoro" onclick={() => start("pomodoro")}>
      &#127813;
    </button>
    <button class={buttonClasses} type="button" title="Stopwatch" onclick={() => start("stopwatch")}>
      &#9201;
    </button>
    <button class={buttonClasses} type="button" title="Timer" onclick={() => (choosingDuration = true)}>
      &#9203;
    </button>
  {/if}
</div>
