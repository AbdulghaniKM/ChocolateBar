<script lang="ts">
  import LoaderCircle from "@lucide/svelte/icons/loader-circle";
  import Plus from "@lucide/svelte/icons/plus";
  import Rocket from "@lucide/svelte/icons/rocket";
  import type { ShortcutGroup } from "../../lib/contracts";
  import { island } from "../../lib/island.svelte";
  import { openSettings, sourceCommand } from "../../lib/ipc";
  import { settingsState } from "../../lib/settings.svelte";
  import { useStream } from "../../lib/stream.svelte";
  import LayoutGlyph from "./LayoutGlyph.svelte";
  import { avatarStyle, initials } from "./appAvatar";

  interface LaunchProgress {
    groupId: string;
    done: number;
    total: number;
    finished: boolean;
  }

  const COLLAPSE_AFTER_LAUNCH_MS = 900;
  const groups = $derived(settingsState.value.shortcuts.groups);
  const progress = useStream<LaunchProgress | null>("launcher", null);
  let failure = $state("");

  const running = $derived(progress.value && !progress.value.finished ? progress.value : null);

  $effect(() => {
    if (!progress.value?.finished) return;
    const timer = setTimeout(() => island.setExpanded(false), COLLAPSE_AFTER_LAUNCH_MS);
    return () => clearTimeout(timer);
  });

  async function launch(group: ShortcutGroup) {
    failure = "";
    try {
      await sourceCommand("launcher", "launch", { id: group.id });
    } catch (error) {
      failure = String(error);
    }
  }
</script>

<div class="flex h-full flex-col gap-3">
  {#if groups.length === 0}
    <div class="flex flex-1 flex-col items-center justify-center gap-3 text-center">
      <span class="grid size-12 place-items-center rounded-2xl bg-accent/12 text-accent"><Rocket class="size-6" /></span>
      <div class="flex flex-col gap-1">
        <p class="text-[0.9375rem] font-semibold">Open your setup in one click</p>
        <p class="max-w-[19rem] text-[0.8125rem] text-island-muted">
          Group apps like Discord and a game, or Chrome on the left and Word on the right.
        </p>
      </div>
      <button
        class="inline-flex h-9 items-center gap-1.5 rounded-pill bg-accent px-4 text-[0.8125rem] font-medium text-on-accent transition-colors hover:bg-accent-strong"
        type="button"
        onclick={() => void openSettings("shortcuts")}
      >
        <Plus class="size-4" /> Create a shortcut
      </button>
    </div>
  {:else}
    <div class="island-scroll grid min-h-0 flex-1 auto-rows-min grid-cols-2 gap-2 overflow-y-auto pr-1">
      {#each groups as group (group.id)}
        {@const active = running?.groupId === group.id}
        <button
          class="tile group flex flex-col gap-2.5 p-3 text-left transition-[background-color,transform] duration-150 hover:bg-island-hover active:scale-[0.98]"
          type="button"
          disabled={!!running}
          onclick={() => launch(group)}
        >
          <div class="flex items-center">
            {#each group.apps.slice(0, 4) as app, index (app.appId + index)}
              <span
                class="-ml-1.5 grid size-7 place-items-center rounded-lg text-[0.625rem] font-bold ring-2 ring-island first:ml-0"
                style={avatarStyle(app.name)}
                title={app.name}
              >
                {initials(app.name)}
              </span>
            {/each}
            {#if group.apps.length > 4}
              <span class="-ml-1.5 grid size-7 place-items-center rounded-lg bg-island-raised text-[0.625rem] font-semibold text-island-muted ring-2 ring-island">
                +{group.apps.length - 4}
              </span>
            {/if}
            <span class="ml-auto text-island-subtle">
              {#if active}
                <LoaderCircle class="size-4 animate-spin text-accent" />
              {/if}
            </span>
          </div>
          <div class="flex min-w-0 flex-col gap-0.5">
            <span class="truncate text-[0.875rem] font-semibold text-island-text">{group.name || "Untitled"}</span>
            <span class="flex items-center gap-1.5 text-[0.6875rem] text-island-muted">
              {#if active && running}
                Opening {running.done + 1 > running.total ? running.total : running.done + 1} of {running.total}…
              {:else}
                {group.apps.length} {group.apps.length === 1 ? "app" : "apps"}
                {#each group.apps.filter((app) => app.layout !== "none").slice(0, 3) as app, index (index)}
                  <LayoutGlyph layout={app.layout} class="h-2.5 w-3.5 text-accent" />
                {/each}
              {/if}
            </span>
          </div>
        </button>
      {/each}
      <button
        class="flex min-h-[5.5rem] flex-col items-center justify-center gap-1 rounded-2xl border border-dashed border-island-border text-[0.75rem] text-island-muted transition-colors hover:border-island-subtle hover:text-island-text"
        type="button"
        onclick={() => void openSettings("shortcuts")}
      >
        <Plus class="size-4" /> New or edit
      </button>
    </div>
  {/if}
  {#if failure}
    <p class="text-[0.75rem] text-danger" role="alert">{failure}</p>
  {/if}
</div>
