import { SvelteMap } from "svelte/reactivity";
import {
  CLOCK_CARD,
  priorityRank,
  type Activity,
} from "./contracts";
import { settingsState } from "./settings.svelte";
import {
  cycleActivity,
  focusActivity,
  listActivities,
  onActivityRemove,
  onActivityUpsert,
  onIslandFocus,
  onIslandVisibility,
  setExpanded,
  setKeyboardInput,
} from "./ipc";

const COLLAPSE_DELAY_MS = 1500;

export type HubTab = "shortcuts" | "clipboard" | "timer";

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
  showingHub = $state(false);
  hubTab = $state<HubTab>("shortcuts");
  ordered = $derived(sortByRank([...this.activities.values()]));
  focused = $derived(this.focusedId ? (this.activities.get(this.focusedId) ?? null) : null);
  /** The flip order shown in card headers: the clock (when it is the default card) and then the user's card order. */
  ring = $derived.by(() => {
    const { cardOrder, defaultCard } = settingsState.value.general;
    const position = (activity: Activity) => {
      const index = cardOrder.indexOf(activity.kind);
      return index === -1 ? cardOrder.length : index;
    };
    const cards = [...this.activities.values()].sort((a, b) => position(a) - position(b));
    return [...(defaultCard === CLOCK_CARD ? [null] : []), ...cards];
  });
  private collapseTimer: ReturnType<typeof setTimeout> | undefined;

  async start(): Promise<() => void> {
    const unlisteners = await Promise.all([
      onActivityUpsert((activity) => this.activities.set(activity.id, activity)),
      onActivityRemove((id) => this.activities.delete(id)),
      onIslandFocus((id) => {
        this.focusedId = id;
        if (id === null && this.expanded && !this.showingHub) this.hubTab = "timer";
      }),
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
    if (!expanded) {
      this.showingHub = false;
      void setKeyboardInput(false);
    }
    void setExpanded(expanded);
  }

  /** Shows an activity the user just started, leaving the hub so its card is visible straight away. */
  async show(id: string) {
    this.showingHub = false;
    this.focusedId = id;
    await focusActivity(id);
    if (!this.expanded) this.setExpanded(true);
  }

  openHub(tab?: HubTab) {
    if (tab) this.hubTab = tab;
    this.showingHub = true;
    if (!this.expanded) this.setExpanded(true);
  }

  closeHub() {
    this.showingHub = false;
  }

  toggleExpanded() {
    this.setExpanded(!this.expanded);
  }

  scheduleCollapse() {
    this.cancelCollapse();
    const typing = document.activeElement instanceof HTMLTextAreaElement || document.activeElement instanceof HTMLInputElement;
    if (!this.expanded || typing) return;
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
