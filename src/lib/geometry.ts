import type { Activity, ActivityKind, IslandPosition } from "./contracts";
import { setIslandSize } from "./ipc";

export interface Size {
  width: number;
  height: number;
}

export const COMPACT_SIZE: Size = { width: 17, height: 2.5 };
export const IDLE_SIZE: Size = { width: 22, height: 13 };
const DEFAULT_CARD: Size = { width: 28, height: 20 };
const CARD_SIZES: Partial<Record<ActivityKind, Size>> = {
  media: { width: 28, height: 23 },
  osd: { width: 24, height: 14 },
  keyboard: { width: 22, height: 15 },
  battery: { width: 24, height: 17 },
};
const WINDOW_GUTTER_REM = 1;

export const LARGEST_CARD: Size = [DEFAULT_CARD, IDLE_SIZE, ...Object.values(CARD_SIZES)].reduce(
  (largest, size) => ({
    width: Math.max(largest.width, size.width),
    height: Math.max(largest.height, size.height),
  }),
  COMPACT_SIZE,
);

export function cardSize(activity: Activity | null): Size {
  if (!activity) return IDLE_SIZE;
  return CARD_SIZES[activity.kind] ?? DEFAULT_CARD;
}

export const toRem = (size: Size) => ({ width: `${size.width}rem`, height: `${size.height}rem` });

export function alignmentClasses(position: IslandPosition): string {
  const vertical = { top: "items-start", middle: "items-center", bottom: "items-end" }[position.vertical];
  const horizontal = { left: "justify-start", center: "justify-center", right: "justify-end" }[position.horizontal];
  return `${vertical} ${horizontal}`;
}

/** The overlay window is sized from rendered rem, so Windows text scaling enlarges it instead of clipping the card. */
export function syncWindowSize(): () => void {
  let published = "";
  const publish = () => {
    const rem = parseFloat(getComputedStyle(document.documentElement).fontSize) || 16;
    const scale = window.devicePixelRatio;
    const width = Math.ceil((LARGEST_CARD.width + WINDOW_GUTTER_REM * 2) * rem * scale);
    const height = Math.ceil((LARGEST_CARD.height + WINDOW_GUTTER_REM * 2) * rem * scale);
    const key = `${width}x${height}`;
    if (key === published) return;
    published = key;
    void setIslandSize(width, height);
  };
  let pixelRatioQuery = matchMedia(`(resolution: ${window.devicePixelRatio}dppx)`);
  const onPixelRatioChange = () => {
    publish();
    pixelRatioQuery.removeEventListener("change", onPixelRatioChange);
    pixelRatioQuery = matchMedia(`(resolution: ${window.devicePixelRatio}dppx)`);
    pixelRatioQuery.addEventListener("change", onPixelRatioChange);
  };
  pixelRatioQuery.addEventListener("change", onPixelRatioChange);
  const timer = setInterval(publish, 5000);
  publish();
  return () => {
    clearInterval(timer);
    pixelRatioQuery.removeEventListener("change", onPixelRatioChange);
  };
}
