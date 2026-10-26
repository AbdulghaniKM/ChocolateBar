<script lang="ts" module>
  interface InstalledApp {
    name: string;
    appId: string;
  }

  let cached = $state<InstalledApp[] | null>(null);
</script>

<script lang="ts">
  import { onMount } from "svelte";
  import FolderOpen from "@lucide/svelte/icons/folder-open";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Search from "@lucide/svelte/icons/search";
  import { sourceCommand } from "../../lib/ipc";
  import { avatarStyle, initials } from "../hub/appAvatar";

  let { onpick }: { onpick: (app: InstalledApp) => void } = $props();

  let query = $state("");
  let path = $state("");
  let loading = $state(false);
  let failure = $state("");

  const matches = $derived(
    (cached ?? []).filter((app) => app.name.toLowerCase().includes(query.trim().toLowerCase())).slice(0, 80),
  );

  async function load(refresh = false) {
    loading = true;
    failure = "";
    try {
      cached = await sourceCommand<InstalledApp[]>("launcher", "apps", { refresh });
    } catch (error) {
      failure = String(error);
    } finally {
      loading = false;
    }
  }

  function addPath() {
    const trimmed = path.trim().replace(/^"|"$/g, "");
    if (!trimmed) return;
    const name = trimmed.split(/[\\/]/).pop()?.replace(/\.(exe|lnk|bat|cmd)$/i, "") || trimmed;
    onpick({ name, appId: trimmed });
    path = "";
  }

  onMount(() => {
    if (!cached) void load();
  });
</script>

<div class="flex flex-col gap-3 rounded-xl border border-island-border bg-island p-3">
  <div class="flex items-center gap-2">
    <label class="relative flex-1">
      <span class="sr-only">Search installed apps</span>
      <Search class="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-island-subtle" />
      <!-- svelte-ignore a11y_autofocus -->
      <input class="w-full !pl-9" type="search" placeholder="Search installed apps" bind:value={query} autofocus />
    </label>
    <button
      class="grid size-9 place-items-center rounded-lg text-island-muted transition-colors hover:bg-island-raised hover:text-island-text"
      type="button"
      aria-label="Refresh app list"
      title="Refresh app list"
      onclick={() => load(true)}
    >
      <RefreshCw class="size-4 {loading ? 'animate-spin' : ''}" />
    </button>
  </div>
  <div class="flex max-h-[15rem] flex-col gap-0.5 overflow-y-auto">
    {#if loading && !cached}
      <p class="p-3 text-sm text-island-muted">Reading the Start menu…</p>
    {:else if failure}
      <p class="p-3 text-sm text-danger">{failure}</p>
    {:else if matches.length === 0}
      <p class="p-3 text-sm text-island-muted">No app matches “{query}”. Add it by path below.</p>
    {:else}
      {#each matches as app (app.appId)}
        <button
          class="flex items-center gap-3 rounded-lg px-2 py-1.5 text-left text-sm transition-colors hover:bg-island-raised"
          type="button"
          onclick={() => onpick(app)}
        >
          <span class="grid size-7 shrink-0 place-items-center rounded-lg text-[0.625rem] font-bold" style={avatarStyle(app.name)}>
            {initials(app.name)}
          </span>
          <span class="truncate">{app.name}</span>
        </button>
      {/each}
    {/if}
  </div>
  <form class="flex items-center gap-2 border-t border-island-border pt-3" onsubmit={(event) => (event.preventDefault(), addPath())}>
    <FolderOpen class="size-4 shrink-0 text-island-subtle" />
    <input class="min-w-0 flex-1" placeholder="Or paste a path, e.g. C:\Games\game.exe" bind:value={path} />
    <button
      class="h-9 rounded-lg bg-island-raised px-3 text-sm transition-colors hover:bg-island-hover disabled:opacity-40"
      type="submit"
      disabled={!path.trim()}
    >
      Add
    </button>
  </form>
</div>
