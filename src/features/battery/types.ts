export type BatteryEventName = "pluggedIn" | "unplugged" | "low";

export interface BatteryView {
  percent: number;
  charging: boolean;
  pluggedIn: boolean;
  full: boolean;
  low: boolean;
  minutesRemaining: number | null;
  event: BatteryEventName | null;
}
