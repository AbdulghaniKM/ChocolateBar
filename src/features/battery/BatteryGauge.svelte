<script lang="ts">
  import Zap from "@lucide/svelte/icons/zap";
  let {
    percent,
    charging,
    low,
    large = false,
  }: { percent: number; charging: boolean; low: boolean; large?: boolean } = $props();

  const tone = $derived(low ? "low" : charging ? "charging" : "normal");
</script>

<div class="gauge {tone}" class:large aria-label="Battery {percent}%">
  <div class="shell">
    <div class="fill" style:transform="scaleX({percent / 100})"></div>
    {#if charging}
      <div class="sweep"></div>
      <span class="bolt"><Zap class="bolt-icon" fill="currentColor" strokeWidth={0} /></span>
    {/if}
  </div>
  <div class="cap"></div>
</div>

<style>
  .gauge {
    --gauge-color: var(--color-island-text);
    display: flex;
    align-items: center;
    flex-shrink: 0;
    width: 1.75rem;
    height: 0.875rem;
  }
  .gauge.large {
    width: 8rem;
    height: 3.75rem;
  }
  .gauge.charging {
    --gauge-color: var(--color-success);
  }
  .gauge.low {
    --gauge-color: var(--color-danger);
  }
  .shell {
    position: relative;
    flex: 1;
    height: 100%;
    overflow: hidden;
    border-radius: 0.3rem;
    border: 0.125rem solid color-mix(in srgb, var(--gauge-color) 70%, transparent);
    background: rgb(255 255 255 / 0.06);
  }
  .large .shell {
    border-radius: 0.8rem;
    border-width: 0.19rem;
  }
  .fill {
    width: 100%;
    height: 100%;
    transform-origin: left center;
    background: linear-gradient(90deg, color-mix(in srgb, var(--gauge-color) 75%, black), var(--gauge-color));
    transition: transform 600ms cubic-bezier(0.32, 0.72, 0, 1);
  }
  .charging .fill {
    animation: charge-breathe 1.8s ease-in-out infinite;
  }
  .low .fill {
    animation: low-blink 1.1s ease-in-out infinite;
  }
  .sweep {
    position: absolute;
    inset: 0;
    background: linear-gradient(100deg, transparent 30%, rgb(255 255 255 / 0.55) 50%, transparent 70%);
    transform: translateX(-100%);
    animation: charge-sweep 1.9s cubic-bezier(0.4, 0, 0.2, 1) infinite;
  }
  .bolt {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-on-accent);
    filter: drop-shadow(0 0 0.15rem rgb(255 255 255 / 0.4));
  }
  .bolt :global(.bolt-icon) {
    width: 0.55rem;
    height: 0.55rem;
  }
  .large .bolt :global(.bolt-icon) {
    width: 1.75rem;
    height: 1.75rem;
  }
  .cap {
    width: 0.1875rem;
    height: 40%;
    margin-left: 0.0625rem;
    border-radius: 0 0.1rem 0.1rem 0;
    background: color-mix(in srgb, var(--gauge-color) 70%, transparent);
  }
  .large .cap {
    width: 0.4rem;
    margin-left: 0.15rem;
  }

  @keyframes charge-sweep {
    to {
      transform: translateX(100%);
    }
  }
  @keyframes charge-breathe {
    50% {
      filter: brightness(1.35);
    }
  }
  @keyframes low-blink {
    50% {
      opacity: 0.45;
    }
  }
</style>
