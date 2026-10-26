export interface SessionView {
  view: "session";
  sessionId: string;
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

export type ToolStatus = "running" | "done" | "error";

export type ChatItem =
  | { kind: "user"; text: string }
  | { kind: "assistant"; text: string }
  | { kind: "tool"; id: string; name: string; summary: string; status: ToolStatus }
  | { kind: "interrupted" };

export interface Todo {
  content: string;
  status: "pending" | "in_progress" | "completed" | string;
}

export interface Conversation {
  items: ChatItem[];
  todos: Todo[];
}
