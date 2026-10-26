import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Activity, HitRect, Settings, SettingsPatch } from "./contracts";

export const events = {
  activityUpsert: "activity://upsert",
  activityRemove: "activity://remove",
  islandFocus: "island://focus",
  islandVisibility: "island://visibility",
  settingsChanged: "settings://changed",
} as const;

export const streamEvent = (source: string) => `stream://${source}`;

export const listActivities = () => invoke<Activity[]>("list_activities");

export const activityAction = (id: string, action: string, value?: unknown) =>
  invoke<void>("activity_action", { id, action, value: value ?? null });

export const sourceCommand = <T = unknown>(
  source: string,
  command: string,
  payload: unknown = null,
) => invoke<T>("source_command", { source, command, payload });

export const cycleActivity = (direction: 1 | -1) =>
  invoke<void>("cycle_activity", { direction });

export const setHitRegions = (rects: HitRect[]) =>
  invoke<void>("set_hit_regions", { rects });

export const setIslandSize = (width: number, height: number) =>
  invoke<void>("set_island_size", { width, height });

export const setExpanded = (expanded: boolean) =>
  invoke<void>("set_expanded", { expanded });

export const getSettings = () => invoke<Settings>("get_settings");

export const updateSettings = (patch: SettingsPatch) =>
  invoke<Settings>("update_settings", { patch });

export const openSettings = () => invoke<void>("open_settings");

const onEvent = <T>(event: string, handler: (payload: T) => void): Promise<UnlistenFn> =>
  listen<T>(event, (message) => handler(message.payload));

export const onActivityUpsert = (handler: (activity: Activity) => void) =>
  onEvent<Activity>(events.activityUpsert, handler);

export const onActivityRemove = (handler: (id: string) => void) =>
  onEvent<string>(events.activityRemove, handler);

export const onIslandFocus = (handler: (id: string | null) => void) =>
  onEvent<string | null>(events.islandFocus, handler);

export const onIslandVisibility = (handler: (visible: boolean) => void) =>
  onEvent<boolean>(events.islandVisibility, handler);

export const onSettingsChanged = (handler: (settings: Settings) => void) =>
  onEvent<Settings>(events.settingsChanged, handler);

export const onStream = <T>(source: string, handler: (payload: T) => void) =>
  onEvent<T>(streamEvent(source), handler);
