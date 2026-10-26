<script lang="ts">
  import type { Snippet } from "svelte";

  let { title, description, children }: { title: string; description?: string; children: Snippet } = $props();
</script>

<section class="flex flex-col gap-4 rounded-2xl border border-island-border bg-island-raised/60 p-5">
  <header class="flex flex-col gap-1">
    <h3 class="text-[0.9375rem] font-semibold text-island-text">{title}</h3>
    {#if description}
      <p class="text-[0.8125rem] text-island-muted">{description}</p>
    {/if}
  </header>
  {@render children()}
</section>
