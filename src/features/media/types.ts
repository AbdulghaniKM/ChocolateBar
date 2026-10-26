export interface LyricLine {
  timeMs: number;
  text: string;
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
}
