import type { Activity, ActivityKind } from "../lib/contracts";

const KIND_LABELS: Record<ActivityKind, string> = {
  media: "Now playing",
  osd: "Volume",
  privacy: "Privacy",
  buds: "Galaxy Buds",
  bluetooth: "Bluetooth",
  battery: "Battery",
  keyboard: "Keyboard",
  download: "Download",
  network: "Network",
  claude: "Claude Code",
  notification: "Notification",
  ocr: "Text capture",
  shelf: "Shelf",
  sports: "Live score",
  timer: "Timer",
  weather: "Weather",
  game: "Games",
};

export function cardLabel(activity: Activity): string {
  const expanded = activity.expanded as { source?: { app?: string }; kind?: string } | null;
  if (activity.kind === "media" && expanded?.source?.app) return expanded.source.app;
  if (activity.kind === "osd" && expanded?.kind === "brightness") return "Brightness";
  return KIND_LABELS[activity.kind];
}
