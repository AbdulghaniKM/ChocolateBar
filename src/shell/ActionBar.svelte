<script lang="ts">
  import type { ActionSpec, ActionStyle } from "../lib/contracts";
  import { activityAction } from "../lib/ipc";
  import Glyph from "./Glyph.svelte";

  let { activityId, actions }: { activityId: string; actions: ActionSpec[] } = $props();

  const styleClasses: Record<ActionStyle, string> = {
    primary: "bg-accent text-on-accent hover:bg-accent-strong",
    neutral: "bg-island-raised text-island-text hover:bg-island-hover",
    danger: "bg-danger/15 text-danger hover:bg-danger/25",
  };
  const buttonBase =
    "inline-flex h-9 items-center justify-center gap-1.5 rounded-pill px-4 text-[0.8125rem] font-medium transition-[background-color,transform] duration-150 active:scale-[0.97]";

  const send = (action: ActionSpec, value?: unknown) =>
    void activityAction(activityId, action.id, value);

  const sendSliderValue = (action: ActionSpec, event: Event) =>
    send(action, Number((event.currentTarget as HTMLInputElement).value));
</script>

{#if actions.length > 0}
  <div class="flex flex-wrap items-center gap-2 px-4 pt-1 pb-4">
    {#each actions as action (action.id)}
      {#if action.control.type === "slider"}
        <label class="flex h-9 min-w-[10rem] flex-1 items-center gap-2.5 rounded-pill bg-island-raised px-3.5 text-island-muted">
          <Glyph glyph={action.icon} class="size-4" />
          <span class="sr-only">{action.label}</span>
          <input
            class="w-full accent-accent"
            type="range"
            min={action.control.min}
            max={action.control.max}
            step={action.control.step}
            value={action.control.value}
            oninput={(event) => sendSliderValue(action, event)}
          />
        </label>
      {:else if action.control.type === "toggle"}
        <button
          class="{buttonBase} {action.control.on ? styleClasses.primary : styleClasses.neutral}"
          type="button"
          aria-pressed={action.control.on}
          onclick={() => send(action, action.control.type === "toggle" && !action.control.on)}
        >
          <Glyph glyph={action.icon} class="size-4" />
          {action.label}
        </button>
      {:else}
        <button class="{buttonBase} {styleClasses[action.style]}" type="button" onclick={() => send(action)}>
          <Glyph glyph={action.icon} class="size-4" />
          {action.label}
        </button>
      {/if}
    {/each}
  </div>
{/if}
