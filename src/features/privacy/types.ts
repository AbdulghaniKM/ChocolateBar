export interface PrivacyExpanded {
  kind: "mic" | "camera" | "mute";
  apps: string[];
  muted: boolean;
}
