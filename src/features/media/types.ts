export interface LyricLine {
  timeMs: number;
  text: string;
}

export type RepeatMode = "off" | "track" | "list";

export interface MediaSource {
  id: string;
  app: string;
  kind: "spotify" | "browser" | "app";
  playing: boolean;
}

export interface MediaExpanded {
  title: string;
  artist: string;
  album: string;
  art: string | null;
  playing: boolean;
  positionMs: number;
  durationMs: number;
  capturedAtMs: number;
  rate: number;
  canGoPrevious: boolean;
  canGoNext: boolean;
  lyrics: LyricLine[];
  shuffle: boolean | null;
  repeat: RepeatMode | null;
  source: MediaSource;
  sources: MediaSource[];
}
