import { defaultSettings, type Settings, type SettingsPatch } from "./contracts";
import { getSettings, onSettingsChanged, updateSettings } from "./ipc";

class SettingsState {
  value = $state<Settings>(structuredClone(defaultSettings));

  async start(): Promise<() => void> {
    const unlisten = await onSettingsChanged((next) => (this.value = next));
    this.value = await getSettings();
    return unlisten;
  }

  async update(patch: SettingsPatch) {
    this.value = await updateSettings(patch);
  }
}

export const settingsState = new SettingsState();
