export type Priority = "ambient" | "normal" | "high" | "critical";
export const CLOCK_CARD = "clock";
export const DEFAULT_CARD_ORDER = [
  "network",
  "media",
  "claude",
  "timer",
  "weather",
  "battery",
  "bluetooth",
  "buds",
  "download",
  "game",
  "sports",
  "privacy",
  "osd",
  "keyboard",
  "notification",
  "ocr",
  "shelf",
] as const;

export const ACTIVITY_KINDS = [
  "media",
  "osd",
  "privacy",
  "buds",
  "bluetooth",
  "battery",
  "keyboard",
  "download",
  "network",
  "claude",
  "notification",
  "ocr",
  "shelf",
  "sports",
  "timer",
  "weather",
  "game",
] as const;

export type ActivityKind = (typeof ACTIVITY_KINDS)[number];

export interface CompactView {
  icon: string;
  title: string;
  subtitle: string | null;
  progress: number | null;
  accent: string | null;
  badge: string | null;
}

export type ActionStyle = "primary" | "neutral" | "danger";

export type ActionControl =
  | { type: "button" }
  | { type: "toggle"; on: boolean }
  | { type: "slider"; min: number; max: number; step: number; value: number };

export interface ActionSpec {
  id: string;
  label: string;
  icon: string | null;
  style: ActionStyle;
  control: ActionControl;
}

export interface Activity<Expanded = unknown> {
  id: string;
  kind: ActivityKind;
  priority: Priority;
  compact: CompactView;
  expanded: Expanded;
  actions: ActionSpec[];
  alert: boolean;
  ttlMs: number | null;
  updatedAtMs: number;
}

export interface HitRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface TeamRef {
  sport: string;
  league: string;
  id: string;
  name: string;
}

export type VerticalAnchor = "top" | "middle" | "bottom";
export type HorizontalAnchor = "left" | "center" | "right";
export interface IslandPosition {
  vertical: VerticalAnchor;
  horizontal: HorizontalAnchor;
  offsetX: number;
  offsetY: number;
}
export type WindowLayout =
  | "none"
  | "maximize"
  | "left"
  | "right"
  | "top-left"
  | "top-right"
  | "bottom-left"
  | "bottom-right";
export interface ShortcutApp {
  appId: string;
  name: string;
  layout: WindowLayout;
}
export interface ShortcutGroup {
  id: string;
  name: string;
  apps: ShortcutApp[];
}
export interface Settings {
  general: {
    hideInFullscreen: boolean;
    autostart: boolean;
    position: IslandPosition;
    defaultCard: string;
    cardOrder: string[];
  };
  hotkeys: { micMute: string; ocr: string };
  sources: Record<string, boolean>;
  media: { lyrics: boolean; visualizer: boolean };
  osd: { replaceVolume: boolean; brightness: boolean };
  claude: { port: number; token: string };
  sports: { leagues: string[]; favoriteTeams: TeamRef[] };
  timer: { workMin: number; shortMin: number; longMin: number };
  weather: { lat: number | null; lon: number | null; city: string | null; units: string };
  ocr: { autoClipboard: boolean; language: string };
  network: { showSpeed: boolean };
  shortcuts: { groups: ShortcutGroup[] };
}

export type SettingsPatch = { [K in keyof Settings]?: Partial<Settings[K]> };

export const defaultSettings: Settings = {
  general: {
    hideInFullscreen: true,
    autostart: false,
    position: { vertical: "bottom", horizontal: "center", offsetX: 0, offsetY: 0 },
    defaultCard: "clock",
    cardOrder: [...DEFAULT_CARD_ORDER],
  },
  hotkeys: { micMute: "Ctrl+Alt+M", ocr: "Ctrl+Alt+O" },
  sources: {},
  media: { lyrics: true, visualizer: true },
  osd: { replaceVolume: true, brightness: true },
  claude: { port: 47653, token: "" },
  sports: {
    leagues: ["eng.1", "esp.1", "uefa.champions", "nba"],
    favoriteTeams: [],
  },
  timer: { workMin: 25, shortMin: 5, longMin: 15 },
  weather: { lat: null, lon: null, city: null, units: "metric" },
  ocr: { autoClipboard: false, language: "auto" },
  network: { showSpeed: true },
  shortcuts: { groups: [] },
};

export const priorityRank: Record<Priority, number> = {
  ambient: 0,
  normal: 1,
  high: 2,
  critical: 3,
};
