<script lang="ts">
  import SourceToggle from "../battery/SourceToggle.svelte";
</script>

<section class="flex flex-col gap-4">
  <h2 class="text-lg font-medium">Galaxy Buds</h2>
  <SourceToggle
    source="buds"
    label="Connect to Galaxy Buds"
    hint="Per-bud and case battery plus ANC, Ambient and Off control. The Galaxy Wearable app must be closed."
  />
</section>
