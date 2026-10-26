<script lang="ts">
  import ArrowLeft from "@lucide/svelte/icons/arrow-left";
  import Brain from "@lucide/svelte/icons/brain";
  import Hourglass from "@lucide/svelte/icons/hourglass";
  import Timer from "@lucide/svelte/icons/timer";
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
    "inline-flex h-9 min-w-9 items-center justify-center gap-1.5 rounded-pill bg-island-raised px-3.5 text-[0.8125rem] font-medium text-island-text transition-[transform,background-color] duration-150 hover:bg-island-hover active:scale-95";
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
      <ArrowLeft class="size-4" />
    </button>
  {:else}
    <button class={buttonClasses} type="button" onclick={() => start("pomodoro")}>
      <Brain class="size-4 text-accent" /> Focus
    </button>
    <button class={buttonClasses} type="button" onclick={() => start("stopwatch")}>
      <Timer class="size-4 text-accent" /> Stopwatch
    </button>
    <button class={buttonClasses} type="button" onclick={() => (choosingDuration = true)}>
      <Hourglass class="size-4 text-accent" /> Timer
    </button>
  {/if}
</div>
