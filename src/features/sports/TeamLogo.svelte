<script lang="ts">
  let {
    logo,
    name,
    class: className = "size-5",
  }: { logo: string | null; name: string; class?: string } = $props();

  let failed = $state(false);
  const initials = $derived(name.slice(0, 2).toUpperCase());
</script>

{#if logo && !failed}
  <img
    class="shrink-0 object-contain {className}"
    src={logo}
    alt={name}
    loading="lazy"
    onerror={() => (failed = true)}
  />
{:else}
  <span
    class="flex shrink-0 items-center justify-center rounded-full bg-island-raised text-[0.5rem] font-semibold text-island-muted {className}"
  >
    {initials}
  </span>
{/if}
