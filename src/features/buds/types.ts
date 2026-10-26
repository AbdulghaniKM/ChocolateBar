export type NoiseMode = "off" | "anc" | "ambient" | "adaptive";
export type LinkStatus = "connected" | "connecting" | "inUse" | "unsupported";
export type Placement = "disconnected" | "wearing" | "idle" | "inCase" | "closedCase";

export interface BudReading {
  percent: number | null;
  charging: boolean;
  placement: Placement;
}

export interface CaseReading {
  percent: number | null;
  charging: boolean;
}

export interface BudsStatusView {
  name: string;
  link: LinkStatus;
  left: BudReading;
  right: BudReading;
  case: CaseReading;
  noise: NoiseMode | null;
  noiseControl: boolean;
  fallbackPercent: number | null;
}

export interface BudsEventView {
  name: string;
  connected: boolean;
}

export type BudsExpanded = BudsStatusView | BudsEventView;

export const isStatusView = (view: BudsExpanded): view is BudsStatusView => "link" in view;

export const NOISE_OPTIONS: { mode: NoiseMode; label: string }[] = [
  { mode: "anc", label: "ANC" },
  { mode: "ambient", label: "Ambient" },
  { mode: "off", label: "Off" },
];

export const LOW_BATTERY_PERCENT = 15;
