export interface LapView {
  index: number;
  splitMs: number;
  totalMs: number;
}

export interface TimerView {
  mode: "countdown" | "stopwatch" | "pomodoro" | "alert";
  label: string;
  accent: string;
  running: boolean;
  elapsedMs: number;
  totalMs: number | null;
  laps: LapView[];
  session: { completed: number; of: number } | null;
}

const MS_PER_SECOND = 1000;
const STOPWATCH_RING_PERIOD_MS = 60_000;

export const liveElapsedMs = (view: TimerView, receivedAtMs: number, nowMs: number) =>
  view.elapsedMs + (view.running ? Math.max(0, nowMs - receivedAtMs) : 0);

export const remainingMs = (view: TimerView, elapsedMs: number) =>
  Math.max(0, (view.totalMs ?? 0) - elapsedMs);

export const isCountingDown = (view: TimerView) => view.totalMs !== null;

export function ringFraction(view: TimerView, elapsedMs: number) {
  if (isCountingDown(view)) {
    return view.totalMs ? remainingMs(view, elapsedMs) / view.totalMs : 0;
  }
  return (elapsedMs % STOPWATCH_RING_PERIOD_MS) / STOPWATCH_RING_PERIOD_MS;
}

export function formatClock(totalSeconds: number) {
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  const pair = (value: number) => String(value).padStart(2, "0");
  return hours > 0 ? `${hours}:${pair(minutes)}:${pair(seconds)}` : `${pair(minutes)}:${pair(seconds)}`;
}

export function clockTextFor(view: TimerView, elapsedMs: number) {
  const seconds = isCountingDown(view)
    ? Math.ceil(remainingMs(view, elapsedMs) / MS_PER_SECOND)
    : Math.floor(elapsedMs / MS_PER_SECOND);
  return formatClock(seconds);
}

export function formatLap(ms: number) {
  const tenths = Math.floor((ms % MS_PER_SECOND) / 100);
  return `${formatClock(Math.floor(ms / MS_PER_SECOND))}.${tenths}`;
}
