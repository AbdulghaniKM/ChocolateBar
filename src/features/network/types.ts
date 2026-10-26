export type LinkName = "wifi" | "wired" | "cellular" | "offline";

export interface NetworkStatusView {
  link: LinkName;
  label: string;
  ssid: string | null;
  signalBars: number | null;
  vpn: boolean;
  showSpeed: boolean;
  downloadText: string;
  uploadText: string;
  downloadHistory: number[];
  uploadHistory: number[];
}

export interface NetworkEventView {
  kind: "wifiConnected" | "wifiDropped" | "vpnConnected" | "vpnDisconnected";
  message: string;
}

export type NetworkExpanded = NetworkStatusView | NetworkEventView;

export const isEventView = (view: NetworkExpanded): view is NetworkEventView => "message" in view;
