export interface SessionView {
  view: "session";
  state: "started" | "thinking" | "tool" | "waiting" | "done" | "interrupted" | "idle";
  project: string | null;
  statusLine: string;
  tool: string | null;
  permissionMode: string | null;
  model: string | null;
}

export interface PermissionView {
  view: "permission";
  toolName: string;
  inputPreview: string;
  project: string | null;
  permissionMode: string | null;
}

export type ClaudeView = SessionView | PermissionView;

export interface HooksReport {
  hooks: { installed: boolean; events: string[]; settingsPath: string; backupExists: boolean };
  listening: boolean;
  port: number;
}

export interface SessionDefaults {
  permissionMode: string | null;
  model: string | null;
  permissionModes: string[];
  models: string[];
  settingsPath: string;
}

const permissionModeLabels: Record<string, string> = {
  default: "Manual",
  acceptEdits: "Accept edits",
  plan: "Plan",
  auto: "Auto",
  dontAsk: "Don't ask",
  bypassPermissions: "Bypass",
};

export const permissionModeLabel = (mode: string) => permissionModeLabels[mode] ?? mode;

export const modelLabel = (model: string) => model.replace(/^claude-/, "");
