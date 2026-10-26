<script lang="ts">
  import type { ActionSpec, ActionStyle } from "../lib/contracts";
  import { activityAction } from "../lib/ipc";

  let { activityId, actions }: { activityId: string; actions: ActionSpec[] } = $props();

  const styleClasses: Record<ActionStyle, string> = {
    primary: "bg-accent text-island",
    neutral: "bg-island-raised text-island-text",
    danger: "bg-danger text-island-text",
  };

  const send = (action: ActionSpec, value?: unknown) =>
    void activityAction(activityId, action.id, value);

  const sendSliderValue = (action: ActionSpec, event: Event) =>
    send(action, Number((event.currentTarget as HTMLInputElement).value));
</script>

{#if actions.length > 0}
  <div class="flex flex-wrap items-center gap-2 px-4 pb-3">
    {#each actions as action (action.id)}
      {#if action.control.type === "slider"}
        <label class="flex min-w-[8rem] flex-1 items-center gap-2 text-xs text-island-muted">
          <span>{action.icon ?? action.label}</span>
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
          class="rounded-pill px-3 py-1 text-xs transition-colors {action.control.on
            ? styleClasses.primary
            : styleClasses.neutral}"
          type="button"
          aria-pressed={action.control.on}
          onclick={() => send(action, action.control.type === "toggle" && !action.control.on)}
        >
          {action.icon ?? ""} {action.label}
        </button>
      {:else}
        <button
          class="rounded-pill px-3 py-1 text-xs transition-opacity hover:opacity-80 {styleClasses[
            action.style
          ]}"
          type="button"
          onclick={() => send(action)}
        >
          {action.icon ?? ""} {action.label}
        </button>
      {/if}
    {/each}
  </div>
{/if}
