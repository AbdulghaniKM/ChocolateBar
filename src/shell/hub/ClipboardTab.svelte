<script lang="ts">
  import { onMount } from "svelte";
  import Check from "@lucide/svelte/icons/check";
  import ClipboardList from "@lucide/svelte/icons/clipboard-list";
  import File from "@lucide/svelte/icons/file";
  import Pin from "@lucide/svelte/icons/pin";
  import PinOff from "@lucide/svelte/icons/pin-off";
  import X from "@lucide/svelte/icons/x";
  import { sourceCommand } from "../../lib/ipc";
  import { useStream } from "../../lib/stream.svelte";

  interface ClipEntry {
    id: number;
    kind: "text" | "image" | "files";
    text: string | null;
    thumbnail: string | null;
    width: number | null;
    height: number | null;
    files: { name: string; path: string }[] | null;
    pinned: boolean;
    capturedAtMs: number;
  }

  const COPIED_FLASH_MS = 1200;
  const live = useStream<ClipEntry[] | null>("clipboard", null);
  let initial = $state<ClipEntry[]>([]);
  let unavailable = $state(false);
  let copiedId = $state<number | null>(null);
  let copiedTimer: ReturnType<typeof setTimeout> | undefined;

  const entries = $derived(live.value ?? initial);
  const unpinnedCount = $derived(entries.filter((entry) => !entry.pinned).length);

  onMount(() => {
    sourceCommand<ClipEntry[]>("clipboard", "list")
      .then((list) => (initial = list))
      .catch(() => (unavailable = true));
    return () => clearTimeout(copiedTimer);
  });

  async function run(command: string, payload: Record<string, unknown> = {}) {
    initial = await sourceCommand<ClipEntry[]>("clipboard", command, payload);
  }

  async function copy(entry: ClipEntry) {
    await run("copy", { id: entry.id });
    copiedId = entry.id;
    clearTimeout(copiedTimer);
    copiedTimer = setTimeout(() => (copiedId = null), COPIED_FLASH_MS);
  }

  const relativeTime = new Intl.RelativeTimeFormat([], { numeric: "auto", style: "narrow" });
  function ago(capturedAtMs: number) {
    const minutes = Math.round((capturedAtMs - Date.now()) / 60000);
    if (minutes > -1) return "now";
    if (minutes > -60) return relativeTime.format(minutes, "minute");
    return relativeTime.format(Math.round(minutes / 60), "hour");
  }

  const iconButton =
    "grid size-7 place-items-center rounded-full text-island-subtle transition-colors hover:bg-island-hover hover:text-island-text";
</script>

<div class="flex h-full flex-col gap-2">
  {#if unavailable}
    <p class="m-auto text-[0.8125rem] text-island-muted">Clipboard history is turned off in Settings.</p>
  {:else if entries.length === 0}
    <div class="m-auto flex flex-col items-center gap-2 text-center">
      <span class="grid size-12 place-items-center rounded-2xl bg-accent/12 text-accent"><ClipboardList class="size-6" /></span>
      <p class="text-[0.9375rem] font-semibold">Nothing copied yet</p>
      <p class="max-w-[19rem] text-[0.8125rem] text-island-muted">
        Copy text, images or files with Ctrl+C and they collect here. Click one to copy it again.
      </p>
    </div>
  {:else}
    <ul class="island-scroll flex min-h-0 flex-1 flex-col gap-1.5 overflow-y-auto pr-1">
      {#each entries as entry (entry.id)}
        <li class="tile group relative flex items-stretch gap-2 transition-colors hover:bg-island-hover">
          <button
            class="flex min-w-0 flex-1 items-center gap-3 rounded-[inherit] p-2.5 text-left"
            type="button"
            title="Copy again"
            onclick={() => copy(entry)}
          >
            {#if entry.kind === "image" && entry.thumbnail}
              <img class="h-12 max-w-[6rem] shrink-0 rounded-lg object-cover ring-1 ring-white/10" src={entry.thumbnail} alt="" />
              <span class="text-[0.75rem] text-island-muted tabular-nums">Image · {entry.width}×{entry.height}</span>
            {:else if entry.kind === "files" && entry.files}
              <span class="grid size-9 shrink-0 place-items-center rounded-lg bg-island-raised text-accent"><File class="size-4" /></span>
              <span class="flex min-w-0 flex-col">
                <span class="truncate text-[0.8125rem] font-medium">{entry.files[0].name}</span>
                <span class="text-[0.6875rem] text-island-muted">
                  {entry.files.length > 1 ? `+${entry.files.length - 1} more` : entry.files[0].path}
                </span>
              </span>
            {:else}
              <span class="line-clamp-3 min-w-0 text-[0.8125rem] leading-snug break-words whitespace-pre-line text-island-text" dir="auto">
                {entry.text}
              </span>
            {/if}
          </button>
          <div class="flex shrink-0 flex-col items-end justify-between py-1.5 pr-1.5">
            <span class="pr-1 text-[0.625rem] text-island-subtle">
              {#if copiedId === entry.id}
                <span class="inline-flex items-center gap-0.5 font-medium text-success"><Check class="size-3" /> Copied</span>
              {:else}
                {ago(entry.capturedAtMs)}
              {/if}
            </span>
            <div class="flex opacity-0 transition-opacity group-focus-within:opacity-100 group-hover:opacity-100 {entry.pinned ? 'opacity-100' : ''}">
              <button
                class="{iconButton} {entry.pinned ? 'text-accent' : ''}"
                type="button"
                aria-label={entry.pinned ? "Unpin" : "Pin"}
                aria-pressed={entry.pinned}
                onclick={() => run("pin", { id: entry.id, pinned: !entry.pinned })}
              >
                {#if entry.pinned}<PinOff class="size-3.5" />{:else}<Pin class="size-3.5" />{/if}
              </button>
              <button class={iconButton} type="button" aria-label="Remove" onclick={() => run("remove", { id: entry.id })}>
                <X class="size-3.5" />
              </button>
            </div>
          </div>
        </li>
      {/each}
    </ul>
    {#if unpinnedCount > 0}
      <button class="self-end text-[0.75rem] text-island-subtle transition-colors hover:text-danger" type="button" onclick={() => run("clear")}>
        Clear unpinned
      </button>
    {/if}
  {/if}
</div>
