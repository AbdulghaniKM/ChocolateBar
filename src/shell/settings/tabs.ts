import type { Component } from "svelte";
import Bell from "@lucide/svelte/icons/bell";
import Bluetooth from "@lucide/svelte/icons/bluetooth";
import BatteryMedium from "@lucide/svelte/icons/battery-medium";
import CloudSun from "@lucide/svelte/icons/cloud-sun";
import Download from "@lucide/svelte/icons/download";
import EighthNotes from "@lucide/svelte/icons/eighth-notes";
import Gamepad2 from "@lucide/svelte/icons/gamepad-2";
import Headphones from "@lucide/svelte/icons/headphones";
import Keyboard from "@lucide/svelte/icons/keyboard";
import Paperclip from "@lucide/svelte/icons/paperclip";
import Rocket from "@lucide/svelte/icons/rocket";
import ScanText from "@lucide/svelte/icons/scan-text";
import ShieldCheck from "@lucide/svelte/icons/shield-check";
import SlidersHorizontal from "@lucide/svelte/icons/sliders-horizontal";
import Sparkle from "@lucide/svelte/icons/sparkle";
import Timer from "@lucide/svelte/icons/timer";
import Trophy from "@lucide/svelte/icons/trophy";
import Volume2 from "@lucide/svelte/icons/volume-2";
import Wifi from "@lucide/svelte/icons/wifi";
import type { ActivityKind } from "../../lib/contracts";

export const GENERAL_TAB = "general";
export const SHORTCUTS_TAB = "shortcuts";

export const TAB_META: Record<
  ActivityKind | typeof GENERAL_TAB | typeof SHORTCUTS_TAB,
  { label: string; icon: Component<{ class?: string }> }
> = {
  general: { label: "General", icon: SlidersHorizontal },
  shortcuts: { label: "Shortcuts", icon: Rocket },
  media: { label: "Media", icon: EighthNotes },
  osd: { label: "Volume & brightness", icon: Volume2 },
  privacy: { label: "Privacy", icon: ShieldCheck },
  buds: { label: "Galaxy Buds", icon: Headphones },
  bluetooth: { label: "Bluetooth", icon: Bluetooth },
  battery: { label: "Battery", icon: BatteryMedium },
  keyboard: { label: "Keyboard", icon: Keyboard },
  download: { label: "Downloads", icon: Download },
  network: { label: "Network", icon: Wifi },
  claude: { label: "Claude Code", icon: Sparkle },
  notification: { label: "Notifications", icon: Bell },
  ocr: { label: "Text capture", icon: ScanText },
  shelf: { label: "File shelf", icon: Paperclip },
  sports: { label: "Sports", icon: Trophy },
  timer: { label: "Timer", icon: Timer },
  weather: { label: "Weather", icon: CloudSun },
  game: { label: "Games", icon: Gamepad2 },
};
