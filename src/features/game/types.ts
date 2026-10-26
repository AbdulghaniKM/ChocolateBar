export interface DownloadView {
  name: string;
  launcher: string;
  fraction: number | null;
  paused: boolean;
  downloadedBytes: number | null;
  totalBytes: number | null;
}

export interface PlayingView {
  name: string;
  startedAtMs: number;
  sessionSeconds: number;
  totalSeconds: number;
}

export interface PlaytimeEntry {
  name: string;
  seconds: number;
}

export const PLAYING_ACTIVITY_ID = "game:playing";

export function formatBytes(bytes: number) {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `${value.toFixed(unit === 0 ? 0 : 1)} ${units[unit]}`;
}

export function formatDuration(totalSeconds: number) {
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  return hours > 0 ? `${hours}h ${String(minutes).padStart(2, "0")}m` : `${minutes}m`;
}
