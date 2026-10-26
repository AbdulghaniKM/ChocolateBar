export interface KeyboardView {
  layoutTag: string;
  layoutName: string;
  capsLock: boolean;
  change: "layout" | "caps";
}
