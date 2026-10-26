<script lang="ts">
  import { onMount } from "svelte";
  import { settingsState } from "../../lib/settings.svelte";
  import { sourceCommand } from "../../lib/ipc";
  import type { HooksReport } from "./claudeView";

  let report = $state<HooksReport>();
  let busy = $state(false);
  let failure = $state("");

  const installed = $derived(report?.hooks.installed ?? false);

  async function run(command: "hooks_status" | "install_hooks" | "uninstall_hooks") {
    busy = true;
    failure = "";
    try {
      report = await sourceCommand<HooksReport>("claude", command);
    } catch (error) {
      failure = String(error);
    } finally {
      busy = false;
    }
  }

  onMount(() => void run("hooks_status"));
</script>

<section class="flex flex-col gap-5">
  <h2 class="text-lg font-medium">Claude Code</h2>

  <div class="flex flex-col gap-3 rounded-2xl border border-island-border bg-island-raised p-4">
    <div class="flex items-center gap-2 text-sm">
      <span class="size-2.5 rounded-full {installed ? 'bg-success' : 'bg-island-muted'}"></span>
      <span>{installed ? "Hooks installed" : "Hooks not installed"}</span>
      {#if report}
        <span class="ml-auto text-xs text-island-muted">
          {report.listening ? `listening on 127.0.0.1:${report.port}` : "server not listening"}
        </span>
      {/if}
    </div>
    {#if report}
      <p class="text-xs break-all text-island-muted">{report.hooks.settingsPath}</p>
      <p class="text-xs text-island-muted">
        {report.hooks.events.length} of 7 events active{report.hooks.backupExists
          ? " · backup saved as settings.json.chocolatebar.bak"
          : ""}
      </p>
    {/if}
    <div class="flex gap-2">
      <button
        class="rounded-pill bg-accent px-4 py-1.5 text-sm font-medium text-island transition-opacity hover:opacity-90 disabled:opacity-50"
        type="button"
        disabled={busy}
        onclick={() => run("install_hooks")}
      >
        {installed ? "Reinstall" : "Install hooks"}
      </button>
      <button
        class="rounded-pill bg-island px-4 py-1.5 text-sm transition-opacity hover:opacity-80 disabled:opacity-50"
        type="button"
        disabled={busy || !installed}
        onclick={() => run("uninstall_hooks")}
      >
        Uninstall
      </button>
      <button
        class="rounded-pill px-4 py-1.5 text-sm text-island-muted hover:text-island-text disabled:opacity-50"
        type="button"
        disabled={busy}
        onclick={() => run("hooks_status")}
      >
        Refresh
      </button>
    </div>
    {#if failure}
      <p class="text-xs text-danger">{failure}</p>
    {/if}
  </div>

  <label class="flex items-center gap-3 text-sm">
    <span class="w-[8rem]">Port</span>
    <input
      class="w-[7rem] rounded-lg bg-island-raised px-3 py-1"
      type="number"
      min="1024"
      max="65535"
      value={settingsState.value.claude.port}
      onchange={(event) =>
        settingsState.update({ claude: { port: Number(event.currentTarget.value) } })}
    />
  </label>
  <p class="text-xs text-island-muted">
    After changing the port, press Reinstall so the hooks point at the new address. Nothing is installed
    until you press the button.
  </p>
</section>
