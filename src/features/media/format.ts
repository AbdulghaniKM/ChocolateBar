export function formatClock(milliseconds: number): string {
  const totalSeconds = Math.max(0, Math.floor(milliseconds / 1000));
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}

export function currentLyricIndex(lines: { timeMs: number }[], positionMs: number): number {
  let index = -1;
  for (let candidate = 0; candidate < lines.length; candidate++) {
    if (lines[candidate].timeMs > positionMs) break;
    index = candidate;
  }
  return index;
}
