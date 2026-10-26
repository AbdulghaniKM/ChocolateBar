<script lang="ts">
  import type { Activity } from "../../lib/contracts";
  import IndeterminateBar from "./IndeterminateBar.svelte";
  import { formatElapsed, type DownloadView } from "./types";

  let { activity }: { activity: Activity } = $props();

  const download = $derived(activity.expanded as DownloadView);
  const stats = $derived([
    { label: "Downloaded", value: download.sizeText },
    { label: download.done ? "Finished in" : "Speed", value: download.done ? formatElapsed(download.elapsedSeconds) : download.speedText },
    { label: "Elapsed", value: formatElapsed(download.elapsedSeconds) },
  ]);
</script>

<div class="flex h-full flex-col justify-center gap-5 px-7">
  <div class="flex items-center gap-4">
    <span
      class="flex size-[3.5rem] shrink-0 items-center justify-center rounded-2xl border border-white/10 bg-white/5 text-[1.75rem] backdrop-blur"
    >
      {activity.compact.icon}
    </span>
    <div class="flex min-w-0 flex-col gap-0.5">
      <span class="truncate text-[1rem] font-medium text-island-text">{download.fileName}</span>
      <span class="text-[0.75rem] {download.done ? 'text-success' : 'text-island-muted'}">
        {download.done ? "Download complete" : "Downloading - total size unknown"}
      </span>
    </div>
  </div>
  <IndeterminateBar done={download.done} tall />
  <div class="grid grid-cols-3 gap-2">
    {#each stats as stat (stat.label)}
      <div class="flex flex-col gap-0.5 rounded-xl border border-white/10 bg-white/5 px-3 py-2">
        <span class="text-[0.625rem] tracking-wider text-island-muted uppercase">{stat.label}</span>
        <span class="text-[0.875rem] font-medium tabular-nums text-island-text">{stat.value}</span>
      </div>
    {/each}
  </div>
</div>
