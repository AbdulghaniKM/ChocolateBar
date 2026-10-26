<script lang="ts">
  import { iconForGlyph } from "../lib/glyphs";

  let {
    glyph,
    class: className = "size-4",
    strokeWidth = 2,
  }: { glyph: string | null | undefined; class?: string; strokeWidth?: number } = $props();

  const Icon = $derived(iconForGlyph(glyph));
</script>

{#if Icon}
  <Icon class="shrink-0 {className}" {strokeWidth} aria-hidden="true" />
{:else if glyph}
  <span class="inline-flex shrink-0 items-center justify-center leading-none {className}" aria-hidden="true">{glyph}</span>
{/if}
