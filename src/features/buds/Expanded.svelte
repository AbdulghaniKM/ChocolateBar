<script lang="ts">
  import Glyph from "../../shell/Glyph.svelte";
  import type { Activity } from "../../lib/contracts";
  import { activityAction } from "../../lib/ipc";
  import BudGauge from "./BudGauge.svelte";
  import NoiseControl from "./NoiseControl.svelte";
  import { isStatusView, type BudsExpanded, type LinkStatus, type NoiseMode } from "./types";

  let { activity }: { activity: Activity } = $props();

  const NOTICES: Record<Exclude<LinkStatus, "connected">, string> = {
    connecting: "Connecting to your buds...",
    inUse: "In use by another app - close the Galaxy Wearable app to enable controls.",
    unsupported: "Controls are not available for this model - showing battery only.",
  };

  const view = $derived(activity.expanded as BudsExpanded);
  const status = $derived(isStatusView(view) ? view : null);
  const notice = $derived(status && status.link !== "connected" ? NOTICES[status.link] : null);
  const isOutOfEar = (placement: string) => placement === "inCase" || placement === "closedCase";

  function selectNoise(mode: NoiseMode) {
    void activityAction(activity.id, "noise", mode);
  }
</script>

{#if status}
  <div class="flex h-full flex-col justify-center gap-5 px-7">
    <div class="flex items-center gap-3">
      <span
        class="flex size-[2.5rem] items-center justify-center tile rounded-xl text-accent"
      >
        <Glyph glyph={activity.compact.icon} class="size-5" />
      </span>
      <div class="flex min-w-0 flex-col leading-tight">
        <span class="truncate text-[1rem] font-medium text-island-text">{status.name}</span>
        <span class="text-[0.6875rem] text-island-muted">{notice ? "Limited" : "Connected"}</span>
      </div>
    </div>

    {#if status.link === "connected"}
      <div class="flex items-start justify-around">
        <BudGauge label="Left" percent={status.left.percent} charging={status.left.charging} dimmed={status.left.placement === "disconnected"} />
        <BudGauge label="Case" percent={status.case.percent} charging={status.case.charging} dimmed={status.case.percent === null} />
        <BudGauge label="Right" percent={status.right.percent} charging={status.right.charging} dimmed={status.right.placement === "disconnected"} />
      </div>
      {#if status.noiseControl}
        <NoiseControl current={status.noise} onselect={selectNoise} />
      {:else if status.left.placement !== "disconnected" || status.right.placement !== "disconnected"}
        <p class="text-center text-[0.6875rem] text-island-muted">
          {isOutOfEar(status.left.placement) && isOutOfEar(status.right.placement) ? "In the case" : "In use"}
        </p>
      {/if}
    {:else}
      <div class="flex flex-col items-center gap-3 tile rounded-2xl p-4 text-center">
        {#if status.fallbackPercent !== null}
          <span class="text-[2rem] leading-none font-semibold tabular-nums text-island-text">{status.fallbackPercent}%</span>
        {/if}
        <span class="text-[0.75rem] text-island-muted">{notice}</span>
      </div>
    {/if}
  </div>
{:else}
  <div class="flex h-full flex-col items-center justify-center gap-3 px-6">
    <Glyph glyph={activity.compact.icon} class="size-12 text-accent" strokeWidth={1.75} />
    <span class="text-[1.125rem] font-medium text-island-text">{activity.compact.title}</span>
    <span class="text-[0.75rem] text-island-muted">{activity.compact.subtitle}</span>
  </div>
{/if}
