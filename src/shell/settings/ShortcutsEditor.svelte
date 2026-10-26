<script lang="ts">
  import Play from "@lucide/svelte/icons/play";
  import Plus from "@lucide/svelte/icons/plus";
  import Trash from "@lucide/svelte/icons/trash";
  import X from "@lucide/svelte/icons/x";
  import type { ShortcutApp, ShortcutGroup, WindowLayout } from "../../lib/contracts";
  import { sourceCommand } from "../../lib/ipc";
  import { settingsState } from "../../lib/settings.svelte";
  import { avatarStyle, initials } from "../hub/appAvatar";
  import LayoutGlyph from "../hub/LayoutGlyph.svelte";
  import AppPicker from "./AppPicker.svelte";

  const LAYOUTS: { id: WindowLayout; label: string }[] = [
    { id: "none", label: "Leave as is" },
    { id: "maximize", label: "Maximize" },
    { id: "left", label: "Left half" },
    { id: "right", label: "Right half" },
    { id: "top-left", label: "Top-left quarter" },
    { id: "top-right", label: "Top-right quarter" },
    { id: "bottom-left", label: "Bottom-left quarter" },
    { id: "bottom-right", label: "Bottom-right quarter" },
  ];

  const groups = $derived(settingsState.value.shortcuts.groups);
  let pickingFor = $state<string | null>(null);
  let launchFailure = $state("");

  const save = (next: ShortcutGroup[]) => void settingsState.update({ shortcuts: { groups: next } });
  const updateGroup = (id: string, change: (group: ShortcutGroup) => ShortcutGroup) =>
    save(groups.map((group) => (group.id === id ? change(group) : group)));

  function addGroup() {
    const id = crypto.randomUUID();
    save([...groups, { id, name: `Shortcut ${groups.length + 1}`, apps: [] }]);
    pickingFor = id;
  }

  function addApp(groupId: string, app: { name: string; appId: string }) {
    updateGroup(groupId, (group) => ({ ...group, apps: [...group.apps, { ...app, layout: "none" }] }));
  }

  function updateApp(groupId: string, index: number, patch: Partial<ShortcutApp>) {
    updateGroup(groupId, (group) => ({
      ...group,
      apps: group.apps.map((app, position) => (position === index ? { ...app, ...patch } : app)),
    }));
  }

  function removeApp(groupId: string, index: number) {
    updateGroup(groupId, (group) => ({ ...group, apps: group.apps.filter((_, position) => position !== index) }));
  }

  async function tryGroup(id: string) {
    launchFailure = "";
    try {
      await sourceCommand("launcher", "launch", { id });
    } catch (error) {
      launchFailure = String(error);
    }
  }
</script>

<div class="flex flex-col gap-4">
  <p class="text-[0.8125rem] text-island-muted">
    A shortcut opens several apps at once and can snap each window into place. Launch them from the island's
    hub (the grid button on any card).
  </p>

  {#each groups as group (group.id)}
    <section class="flex flex-col gap-3 rounded-2xl border border-island-border bg-island-raised/60 p-4">
      <div class="flex items-center gap-2">
        <label class="flex-1">
          <span class="sr-only">Shortcut name</span>
          <input
            class="w-full !text-[0.9375rem] font-semibold"
            value={group.name}
            onchange={(event) => updateGroup(group.id, (current) => ({ ...current, name: event.currentTarget.value.trim() }))}
          />
        </label>
        <button
          class="inline-flex h-9 items-center gap-1.5 rounded-lg bg-accent px-3 text-sm font-medium text-on-accent transition-colors hover:bg-accent-strong disabled:opacity-40"
          type="button"
          disabled={group.apps.length === 0}
          onclick={() => tryGroup(group.id)}
        >
          <Play class="size-3.5" fill="currentColor" /> Try it
        </button>
        <button
          class="grid size-9 place-items-center rounded-lg text-island-muted transition-colors hover:bg-danger/15 hover:text-danger"
          type="button"
          aria-label="Delete {group.name}"
          title="Delete shortcut"
          onclick={() => save(groups.filter((candidate) => candidate.id !== group.id))}
        >
          <Trash class="size-4" />
        </button>
      </div>

      {#if group.apps.length > 0}
        <ul class="flex flex-col gap-1.5">
          {#each group.apps as app, index (index)}
            <li class="flex flex-wrap items-center gap-3 rounded-xl bg-island px-3 py-2">
              <span class="grid size-8 shrink-0 place-items-center rounded-lg text-[0.6875rem] font-bold" style={avatarStyle(app.name)}>
                {initials(app.name)}
              </span>
              <span class="min-w-[7rem] flex-1 truncate text-sm" title={app.appId}>{app.name}</span>
              <div class="flex items-center gap-0.5 rounded-lg bg-island-raised/70 p-0.5" role="radiogroup" aria-label="Window layout for {app.name}">
                {#each LAYOUTS as option (option.id)}
                  {@const active = app.layout === option.id}
                  <button
                    class="grid h-7 w-8 place-items-center rounded-md transition-colors
                      {active ? 'bg-accent text-on-accent' : 'text-island-muted hover:bg-island-hover hover:text-island-text'}"
                    type="button"
                    role="radio"
                    aria-checked={active}
                    aria-label={option.label}
                    title={option.label}
                    onclick={() => updateApp(group.id, index, { layout: option.id })}
                  >
                    <LayoutGlyph layout={option.id} />
                  </button>
                {/each}
              </div>
              <button
                class="grid size-7 place-items-center rounded-full text-island-subtle transition-colors hover:text-danger"
                type="button"
                aria-label="Remove {app.name}"
                onclick={() => removeApp(group.id, index)}
              >
                <X class="size-4" />
              </button>
            </li>
          {/each}
        </ul>
      {/if}

      {#if pickingFor === group.id}
        <AppPicker onpick={(app) => addApp(group.id, app)} />
        <button class="self-start text-sm text-island-muted hover:text-island-text" type="button" onclick={() => (pickingFor = null)}>
          Done adding
        </button>
      {:else}
        <button
          class="inline-flex h-9 items-center gap-1.5 self-start rounded-lg bg-island px-3 text-sm text-island-muted transition-colors hover:text-island-text"
          type="button"
          onclick={() => (pickingFor = group.id)}
        >
          <Plus class="size-4" /> Add app
        </button>
      {/if}
    </section>
  {/each}

  <button
    class="inline-flex h-11 items-center justify-center gap-2 rounded-2xl border border-dashed border-island-border text-sm text-island-muted transition-colors hover:border-island-subtle hover:text-island-text"
    type="button"
    onclick={addGroup}
  >
    <Plus class="size-4" /> New shortcut
  </button>
  {#if launchFailure}
    <p class="text-sm text-danger" role="alert">{launchFailure}</p>
  {/if}
  <p class="text-xs text-island-subtle">
    Snapping waits for each app's window, up to 15 seconds. Apps that are already open are brought forward and moved.
  </p>
</div>
