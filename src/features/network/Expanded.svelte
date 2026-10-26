<script lang="ts">
  import Glyph from "../../shell/Glyph.svelte";
  import type { Activity } from "../../lib/contracts";
  import Sparkline from "./Sparkline.svelte";
  import { isEventView, type NetworkExpanded } from "./types";

  let { activity }: { activity: Activity } = $props();

  const view = $derived(activity.expanded as NetworkExpanded);
  const event = $derived(isEventView(view) ? view : null);
  const status = $derived(isEventView(view) ? null : view);
  const bars = $derived(status?.signalBars ?? null);
  const meters = $derived(
    status
      ? [
          { label: "Download", arrow: "↓", text: status.downloadText, history: status.downloadHistory, color: "var(--color-info)" },
          { label: "Upload", arrow: "↑", text: status.uploadText, history: status.uploadHistory, color: "var(--color-success)" },
        ]
      : [],
  );
</script>

{#if event}
  <div class="flex h-full flex-col items-center justify-center gap-3 px-6">
    <span style:color={activity.compact.accent}><Glyph glyph={activity.compact.icon} class="size-12" strokeWidth={1.75} /></span>
    <span class="text-[1.125rem] font-medium text-island-text">{event.message}</span>
  </div>
{:else if status}
  <div class="flex h-full flex-col justify-center gap-4 px-7">
    <div class="flex items-center gap-3">
      <span
        class="flex size-[3rem] items-center justify-center tile rounded-2xl"
        style:color={activity.compact.accent}
      >
        <Glyph glyph={activity.compact.icon} class="size-6" />
      </span>
      <div class="flex min-w-0 flex-1 flex-col gap-0.5">
        <span class="truncate text-[1rem] font-medium text-island-text">{status.label}</span>
        <span class="text-[0.75rem] text-island-muted">
          {status.ssid === null && status.link === "wifi" ? "Wi-Fi connected" : status.link === "offline" ? "No connection" : "Connected"}
        </span>
      </div>
      {#if bars !== null}
        <div class="flex h-[1.125rem] items-end gap-[0.1875rem]" aria-label="Signal {bars} of 5">
          {#each [1, 2, 3, 4, 5] as level (level)}
            <span
              class="w-[0.25rem] rounded-sm transition-colors duration-300 {level <= bars ? 'bg-island-text' : 'bg-island-border'}"
              style:height="{level * 20}%"
            ></span>
          {/each}
        </div>
      {/if}
      {#if status.vpn}
        <span class="rounded-pill border border-berry/40 bg-berry/15 px-2.5 py-0.5 text-[0.6875rem] font-semibold text-berry">
          VPN on
        </span>
      {/if}
    </div>
    {#if status.link !== "offline"}
      <div class="grid grid-cols-2 gap-3">
        {#each meters as meter (meter.label)}
          <div class="flex flex-col gap-2 tile rounded-2xl p-3">
            <div class="flex items-baseline justify-between">
              <span class="text-[0.625rem] tracking-wider text-island-muted uppercase">{meter.label}</span>
              <span class="text-[0.75rem]" style:color={meter.color}>{meter.arrow}</span>
            </div>
            <span class="text-[1.125rem] font-semibold tabular-nums text-island-text">{meter.text}</span>
            <div class="h-[2rem]">
              <Sparkline samples={meter.history} color={meter.color} />
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
{/if}
