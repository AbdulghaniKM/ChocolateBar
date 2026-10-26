import type { MediaExpanded } from "./types";

const TICK_MS = 250;

export function usePlaybackPosition(getMedia: () => MediaExpanded) {
  let now = $state(Date.now());

  $effect(() => {
    const timer = setInterval(() => (now = Date.now()), TICK_MS);
    return () => clearInterval(timer);
  });

  const positionMs = $derived.by(() => {
    const media = getMedia();
    const elapsed = media.playing ? Math.max(0, now - media.capturedAtMs) * media.rate : 0;
    const position = media.positionMs + elapsed;
    return media.durationMs > 0 ? Math.min(position, media.durationMs) : position;
  });

  return {
    get positionMs() {
      return positionMs;
    },
  };
}
