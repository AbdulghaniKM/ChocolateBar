<script lang="ts">
  import { onMount } from "svelte";
  import { sourceCommand } from "../../lib/ipc";

  let status = $state("checking…");
  let listening = $state(false);

  onMount(async () => {
    try {
      const reply = await sourceCommand<{ status: string; listening: boolean }>("notification", "status");
      status = reply.status;
      listening = reply.listening;
    } catch {
      status = "source is not running";
    }
  });
</script>

<section class="flex flex-col gap-4">
  <h2 class="text-lg font-medium">Notification mirror</h2>
  <div class="flex items-center gap-2 rounded-2xl border border-island-border bg-island-raised p-4 text-sm">
    <span class="size-2.5 rounded-full {listening ? 'bg-success' : 'bg-island-muted'}"></span>
    <span class="capitalize">{status}</span>
  </div>
  <p class="text-xs text-island-muted">
    Windows only lets packaged apps read toast notifications. Until ChocolateBar ships with a package
    identity, the mirror stays idle.
  </p>
</section>
