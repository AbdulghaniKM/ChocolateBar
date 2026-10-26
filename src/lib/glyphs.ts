import type { Component } from "svelte";
import AlarmClock from "@lucide/svelte/icons/alarm-clock";
import ArrowDownToLine from "@lucide/svelte/icons/arrow-down-to-line";
import Battery from "@lucide/svelte/icons/battery";
import BatteryCharging from "@lucide/svelte/icons/battery-charging";
import Bell from "@lucide/svelte/icons/bell";
import Bluetooth from "@lucide/svelte/icons/bluetooth";
import Brain from "@lucide/svelte/icons/brain";
import Camera from "@lucide/svelte/icons/camera";
import Check from "@lucide/svelte/icons/check";
import CircleCheck from "@lucide/svelte/icons/circle-check";
import Cloud from "@lucide/svelte/icons/cloud";
import CloudDrizzle from "@lucide/svelte/icons/cloud-drizzle";
import CloudFog from "@lucide/svelte/icons/cloud-fog";
import CloudLightning from "@lucide/svelte/icons/cloud-lightning";
import CloudMoon from "@lucide/svelte/icons/cloud-moon";
import CloudRain from "@lucide/svelte/icons/cloud-rain";
import CloudSnow from "@lucide/svelte/icons/cloud-snow";
import CloudSun from "@lucide/svelte/icons/cloud-sun";
import Coffee from "@lucide/svelte/icons/coffee";
import Copy from "@lucide/svelte/icons/copy";
import EighthNotes from "@lucide/svelte/icons/eighth-notes";
import Flag from "@lucide/svelte/icons/flag";
import Gamepad2 from "@lucide/svelte/icons/gamepad-2";
import Globe from "@lucide/svelte/icons/globe";
import Goal from "@lucide/svelte/icons/goal";
import Headphones from "@lucide/svelte/icons/headphones";
import Hourglass from "@lucide/svelte/icons/hourglass";
import Inbox from "@lucide/svelte/icons/inbox";
import Joystick from "@lucide/svelte/icons/joystick";
import KeyRound from "@lucide/svelte/icons/key-round";
import Keyboard from "@lucide/svelte/icons/keyboard";
import Lock from "@lucide/svelte/icons/lock";
import LockOpen from "@lucide/svelte/icons/lock-open";
import Mic from "@lucide/svelte/icons/mic";
import Moon from "@lucide/svelte/icons/moon";
import Mouse from "@lucide/svelte/icons/mouse";
import Paperclip from "@lucide/svelte/icons/paperclip";
import Pause from "@lucide/svelte/icons/pause";
import Play from "@lucide/svelte/icons/play";
import RotateCcw from "@lucide/svelte/icons/rotate-ccw";
import ScanText from "@lucide/svelte/icons/scan-text";
import SkipForward from "@lucide/svelte/icons/skip-forward";
import Snowflake from "@lucide/svelte/icons/snowflake";
import Sparkle from "@lucide/svelte/icons/sparkle";
import Sun from "@lucide/svelte/icons/sun";
import SunMedium from "@lucide/svelte/icons/sun-medium";
import Thermometer from "@lucide/svelte/icons/thermometer";
import ThermometerSun from "@lucide/svelte/icons/thermometer-sun";
import Timer from "@lucide/svelte/icons/timer";
import Trash from "@lucide/svelte/icons/trash";
import TriangleAlert from "@lucide/svelte/icons/triangle-alert";
import Trophy from "@lucide/svelte/icons/trophy";
import Volume from "@lucide/svelte/icons/volume";
import Volume1 from "@lucide/svelte/icons/volume-1";
import Volume2 from "@lucide/svelte/icons/volume-2";
import VolumeX from "@lucide/svelte/icons/volume-x";
import Wifi from "@lucide/svelte/icons/wifi";
import Wind from "@lucide/svelte/icons/wind";
import X from "@lucide/svelte/icons/x";

export type IconComponent = Component<{ class?: string; strokeWidth?: number; [attribute: string]: unknown }>;

const VARIATION_SELECTOR = new RegExp(String.fromCharCode(0xfe0f), "g");

/** Sources describe icons with emoji/symbol glyphs; the island renders them as one consistent SVG set. */
const glyphIcons: Record<string, IconComponent> = {
  "🎵": EighthNotes,
  "🔇": VolumeX,
  "🔈": Volume,
  "🔉": Volume1,
  "🔊": Volume2,
  "🔆": SunMedium,
  "⚡": BatteryCharging,
  "🔋": Battery,
  "🎧": Headphones,
  "⌨": Keyboard,
  "🖱": Mouse,
  "🎮": Gamepad2,
  "🔵": Bluetooth,
  "📶": Wifi,
  "🌐": Globe,
  "⚠": TriangleAlert,
  "🔒": Lock,
  "🔓": LockOpen,
  "🎙": Mic,
  "📷": Camera,
  "✦": Sparkle,
  "🔐": KeyRound,
  "⬇": ArrowDownToLine,
  "✅": CircleCheck,
  "🕹": Joystick,
  "🔔": Bell,
  "📝": ScanText,
  "📎": Paperclip,
  "📥": Inbox,
  "🗑": Trash,
  "⧉": Copy,
  "✓": Check,
  "✕": X,
  "⏰": AlarmClock,
  "⏳": Hourglass,
  "⏱": Timer,
  "🍅": Brain,
  "☕": Coffee,
  "⏭": SkipForward,
  "⏸": Pause,
  "▶": Play,
  "⚑": Flag,
  "↺": RotateCcw,
  "🏀": Trophy,
  "⚽": Goal,
  "☀": Sun,
  "🌙": Moon,
  "🌤": CloudSun,
  "⛅": CloudSun,
  "☁": Cloud,
  "🌫": CloudFog,
  "🌦": CloudDrizzle,
  "🌧": CloudRain,
  "🌨": CloudSnow,
  "⛈": CloudLightning,
  "🌡": Thermometer,
  "💨": Wind,
  "🥵": ThermometerSun,
  "🥶": Snowflake,
  "☾": CloudMoon,
};

export function iconForGlyph(glyph: string | null | undefined): IconComponent | null {
  if (!glyph) return null;
  return glyphIcons[glyph.replace(VARIATION_SELECTOR, "").trim()] ?? null;
}
