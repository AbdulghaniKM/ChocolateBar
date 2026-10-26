export interface ShelfEntry {
  path: string;
  name: string;
  isDirectory: boolean;
}

export type ShelfView = { view: "drop" } | { view: "files"; files: ShelfEntry[] };
