export interface SessionView {
  view: "session";
  state: "started" | "thinking" | "tool" | "waiting" | "done";
  project: string | null;
  statusLine: string;
  tool: string | null;
}

export interface PermissionView {
  view: "permission";
  toolName: string;
  inputPreview: string;
  project: string | null;
}

export type ClaudeView = SessionView | PermissionView;

export interface HooksReport {
  hooks: { installed: boolean; events: string[]; settingsPath: string; backupExists: boolean };
  listening: boolean;
  port: number;
}
