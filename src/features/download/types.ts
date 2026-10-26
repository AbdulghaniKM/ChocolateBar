export interface DownloadView {
  fileName: string;
  bytesDone: number;
  sizeText: string;
  bytesPerSecond: number;
  speedText: string;
  elapsedSeconds: number;
  done: boolean;
  indeterminate: boolean;
}

export function formatElapsed(totalSeconds: number): string {
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return minutes > 0 ? `${minutes}m ${String(seconds).padStart(2, "0")}s` : `${seconds}s`;
}
