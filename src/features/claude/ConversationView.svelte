<script lang="ts">
  import { onMount, tick } from "svelte";
  import Check from "@lucide/svelte/icons/check";
  import CircleSlash from "@lucide/svelte/icons/circle-slash";
  import LoaderCircle from "@lucide/svelte/icons/loader-circle";
  import X from "@lucide/svelte/icons/x";
  import { sourceCommand } from "../../lib/ipc";
  import { useStream } from "../../lib/stream.svelte";
  import Markdown from "./Markdown.svelte";
  import type { Conversation } from "./claudeView";

  let { sessionId }: { sessionId: string } = $props();

  const NEAR_BOTTOM_PX = 48;
  const live = useStream<{ sessionId: string; conversation: Conversation } | null>("claude", null);
  let loaded = $state<Conversation | null>(null);
  let viewport = $state<HTMLDivElement>();
  let followBottom = true;

  const conversation = $derived(live.value?.sessionId === sessionId ? live.value.conversation : loaded);
  const items = $derived(conversation?.items ?? []);

  onMount(() => {
    sourceCommand<Conversation>("claude", "conversation", { sessionId })
      .then((value) => (loaded = value))
      .catch(() => (loaded = { items: [], todos: [] }));
  });

  $effect(() => {
    void items.length;
    if (!followBottom) return;
    void tick().then(() => viewport?.scrollTo({ top: viewport.scrollHeight }));
  });

  function trackScroll() {
    if (!viewport) return;
    followBottom = viewport.scrollHeight - viewport.scrollTop - viewport.clientHeight < NEAR_BOTTOM_PX;
  }
</script>

<div bind:this={viewport} class="island-scroll h-full overflow-y-auto pr-1" onscroll={trackScroll}>
  {#if conversation === null}
    <p class="py-6 text-center text-[0.8125rem] text-island-subtle">Loading the conversation…</p>
  {:else if items.length === 0}
    <p class="py-6 text-center text-[0.8125rem] text-island-subtle">Messages from this session will appear here.</p>
  {:else}
    <ol class="flex flex-col gap-2.5 py-1">
      {#each items as item, index (index)}
        {#if item.kind === "user"}
          <li class="flex justify-end">
            <p
              class="max-w-[85%] rounded-2xl rounded-br-md bg-island-raised px-3 py-2 text-[0.8125rem] leading-relaxed break-words whitespace-pre-wrap text-island-text"
              dir="auto"
            >
              {item.text}
            </p>
          </li>
        {:else if item.kind === "assistant"}
          <li class="pr-4"><Markdown source={item.text} /></li>
        {:else if item.kind === "tool"}
          <li class="flex min-w-0 items-center gap-2 text-[0.75rem]">
            <span
              class="grid size-4 shrink-0 place-items-center rounded-full
                {item.status === 'error' ? 'bg-danger/15 text-danger' : item.status === 'done' ? 'bg-success/15 text-success' : 'text-accent'}"
            >
              {#if item.status === "running"}
                <LoaderCircle class="size-3.5 animate-spin" />
              {:else if item.status === "error"}
                <X class="size-2.5" strokeWidth={3} />
              {:else}
                <Check class="size-2.5" strokeWidth={3} />
              {/if}
            </span>
            <span class="shrink-0 font-semibold text-island-text">{item.name}</span>
            {#if item.summary}
              <span class="min-w-0 truncate font-mono text-[0.6875rem] text-island-muted">{item.summary}</span>
            {/if}
          </li>
        {:else}
          <li class="flex items-center gap-1.5 text-[0.75rem] text-island-subtle">
            <CircleSlash class="size-3.5" /> Interrupted
          </li>
        {/if}
      {/each}
    </ol>
  {/if}
</div>
