import type { Component } from "svelte";
import type { Activity, ActivityKind } from "../lib/contracts";
import MediaCompact from "./media/Compact.svelte";
import MediaExpanded from "./media/Expanded.svelte";
import MediaSettings from "./media/Settings.svelte";
import OsdCompact from "./osd/Compact.svelte";
import OsdExpanded from "./osd/Expanded.svelte";
import OsdSettings from "./osd/Settings.svelte";
import PrivacyCompact from "./privacy/Compact.svelte";
import PrivacyExpanded from "./privacy/Expanded.svelte";
import PrivacySettings from "./privacy/Settings.svelte";
import BudsCompact from "./buds/Compact.svelte";
import BudsExpanded from "./buds/Expanded.svelte";
import BudsSettings from "./buds/Settings.svelte";
import BluetoothCompact from "./bluetooth/Compact.svelte";
import BluetoothExpanded from "./bluetooth/Expanded.svelte";
import BluetoothSettings from "./bluetooth/Settings.svelte";
import BatteryCompact from "./battery/Compact.svelte";
import BatteryExpanded from "./battery/Expanded.svelte";
import BatterySettings from "./battery/Settings.svelte";
import KeyboardCompact from "./keyboard/Compact.svelte";
import KeyboardExpanded from "./keyboard/Expanded.svelte";
import KeyboardSettings from "./keyboard/Settings.svelte";
import DownloadCompact from "./download/Compact.svelte";
import DownloadExpanded from "./download/Expanded.svelte";
import DownloadSettings from "./download/Settings.svelte";
import NetworkCompact from "./network/Compact.svelte";
import NetworkExpanded from "./network/Expanded.svelte";
import NetworkSettings from "./network/Settings.svelte";
import ClaudeCompact from "./claude/Compact.svelte";
import ClaudeExpanded from "./claude/Expanded.svelte";
import ClaudeSettings from "./claude/Settings.svelte";
import NotificationCompact from "./notification/Compact.svelte";
import NotificationExpanded from "./notification/Expanded.svelte";
import NotificationSettings from "./notification/Settings.svelte";
import OcrCompact from "./ocr/Compact.svelte";
import OcrExpanded from "./ocr/Expanded.svelte";
import OcrSettings from "./ocr/Settings.svelte";
import ShelfCompact from "./shelf/Compact.svelte";
import ShelfExpanded from "./shelf/Expanded.svelte";
import ShelfSettings from "./shelf/Settings.svelte";
import SportsCompact from "./sports/Compact.svelte";
import SportsExpanded from "./sports/Expanded.svelte";
import SportsSettings from "./sports/Settings.svelte";
import TimerCompact from "./timer/Compact.svelte";
import TimerExpanded from "./timer/Expanded.svelte";
import TimerSettings from "./timer/Settings.svelte";
import WeatherCompact from "./weather/Compact.svelte";
import WeatherExpanded from "./weather/Expanded.svelte";
import WeatherSettings from "./weather/Settings.svelte";
import GameCompact from "./game/Compact.svelte";
import GameExpanded from "./game/Expanded.svelte";
import GameSettings from "./game/Settings.svelte";
import TimerLauncher from "./timer/Launcher.svelte";

export interface FeatureEntry {
  Compact?: Component<{ activity: Activity }>;
  Expanded: Component<{ activity: Activity }>;
  Settings?: Component;
  Launcher?: Component;
}

export const registry: Record<ActivityKind, FeatureEntry> = {
  media: { Compact: MediaCompact, Expanded: MediaExpanded, Settings: MediaSettings },
  osd: { Compact: OsdCompact, Expanded: OsdExpanded, Settings: OsdSettings },
  privacy: { Compact: PrivacyCompact, Expanded: PrivacyExpanded, Settings: PrivacySettings },
  buds: { Compact: BudsCompact, Expanded: BudsExpanded, Settings: BudsSettings },
  bluetooth: { Compact: BluetoothCompact, Expanded: BluetoothExpanded, Settings: BluetoothSettings },
  battery: { Compact: BatteryCompact, Expanded: BatteryExpanded, Settings: BatterySettings },
  keyboard: { Compact: KeyboardCompact, Expanded: KeyboardExpanded, Settings: KeyboardSettings },
  download: { Compact: DownloadCompact, Expanded: DownloadExpanded, Settings: DownloadSettings },
  network: { Compact: NetworkCompact, Expanded: NetworkExpanded, Settings: NetworkSettings },
  claude: { Compact: ClaudeCompact, Expanded: ClaudeExpanded, Settings: ClaudeSettings },
  notification: { Compact: NotificationCompact, Expanded: NotificationExpanded, Settings: NotificationSettings },
  ocr: { Compact: OcrCompact, Expanded: OcrExpanded, Settings: OcrSettings },
  shelf: { Compact: ShelfCompact, Expanded: ShelfExpanded, Settings: ShelfSettings },
  sports: { Compact: SportsCompact, Expanded: SportsExpanded, Settings: SportsSettings },
  timer: { Compact: TimerCompact, Expanded: TimerExpanded, Settings: TimerSettings, Launcher: TimerLauncher },
  weather: { Compact: WeatherCompact, Expanded: WeatherExpanded, Settings: WeatherSettings },
  game: { Compact: GameCompact, Expanded: GameExpanded, Settings: GameSettings },
};
