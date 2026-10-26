export type BluetoothEventName = "connected" | "disconnected" | "lowBattery";

export interface BluetoothView {
  name: string;
  deviceKind: "headphones" | "keyboard" | "mouse" | "gamepad" | "speaker" | "generic";
  connected: boolean;
  battery: number | null;
  event: BluetoothEventName | null;
}

export const LOW_BATTERY_PERCENT = 15;
