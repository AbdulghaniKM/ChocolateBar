<script lang="ts">
  import { parseMarkdown, type Inline } from "./markdown";

  let { source }: { source: string } = $props();

  const blocks = $derived(parseMarkdown(source));
</script>

{#snippet inlines(parts: Inline[])}
  {#each parts as part, index (index)}
    {#if part.kind === "code"}
      <code class="rounded bg-island-raised px-1 py-px font-mono text-[0.75rem] text-accent-strong">{part.text}</code>
    {:else if part.kind === "bold"}
      <strong class="font-semibold text-island-text">{part.text}</strong>
    {:else}
      {part.text}
    {/if}
  {/each}
{/snippet}

<div class="flex flex-col gap-1.5 text-[0.8125rem] leading-relaxed break-words text-island-text/90" dir="auto">
  {#each blocks as block, index (index)}
    {#if block.kind === "code"}
      <pre class="island-scroll overflow-x-auto rounded-lg border border-island-border bg-island-raised/70 px-2.5 py-2 font-mono text-[0.71875rem] leading-snug text-island-text">{block.text}</pre>
    {:else if block.kind === "heading"}
      <p class="font-semibold text-island-text">{@render inlines(block.inlines)}</p>
    {:else if block.kind === "list"}
      <svelte:element this={block.ordered ? "ol" : "ul"} class="flex flex-col gap-0.5 pl-4 {block.ordered ? 'list-decimal' : 'list-disc'} marker:text-island-subtle">
        {#each block.items as item, itemIndex (itemIndex)}
          <li>{@render inlines(item)}</li>
        {/each}
      </svelte:element>
    {:else}
      <p>{@render inlines(block.inlines)}</p>
    {/if}
  {/each}
</div>
