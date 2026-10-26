import { SvelteMap } from "svelte/reactivity";
import {
  priorityRank,
  type Activity,
} from "./contracts";
import {
  cycleActivity,
  listActivities,
  onActivityRemove,
  onActivityUpsert,
  onIslandFocus,
  onIslandVisibility,
  setExpanded,
} from "./ipc";

const COLLAPSE_DELAY_MS = 1500;

export function sortByRank(activities: Activity[]): Activity[] {
  return [...activities].sort(
    (a, b) =>
      priorityRank[b.priority] - priorityRank[a.priority] ||
      Number(b.alert) - Number(a.alert) ||
      b.updatedAtMs - a.updatedAtMs,
  );
}

class Island {
  activities = new SvelteMap<string, Activity>();
  focusedId = $state<string | null>(null);
  expanded = $state(false);
  hidden = $state(false);
  ordered = $derived(sortByRank([...this.activities.values()]));
  focused = $derived(this.focusedId ? (this.activities.get(this.focusedId) ?? null) : null);
  private collapseTimer: ReturnType<typeof setTimeout> | undefined;

  async start(): Promise<() => void> {
    const unlisteners = await Promise.all([
      onActivityUpsert((activity) => this.activities.set(activity.id, activity)),
      onActivityRemove((id) => this.activities.delete(id)),
      onIslandFocus((id) => (this.focusedId = id)),
      onIslandVisibility((visible) => (this.hidden = !visible)),
    ]);
    await this.loadSnapshot();
    return () => unlisteners.forEach((unlisten) => unlisten());
  }

  cycle(direction: 1 | -1) {
    void cycleActivity(direction);
  }

  setExpanded(expanded: boolean) {
    this.cancelCollapse();
    this.expanded = expanded;
    void setExpanded(expanded);
  }

  toggleExpanded() {
    this.setExpanded(!this.expanded);
  }

  scheduleCollapse() {
    this.cancelCollapse();
    if (!this.expanded) return;
    this.collapseTimer = setTimeout(() => this.setExpanded(false), COLLAPSE_DELAY_MS);
  }

  cancelCollapse() {
    clearTimeout(this.collapseTimer);
  }

  private async loadSnapshot() {
    const snapshot = await listActivities();
    snapshot.forEach((activity) => this.activities.set(activity.id, activity));
  }
}

export const island = new Island();
