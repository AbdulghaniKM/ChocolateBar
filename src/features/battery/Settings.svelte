<script lang="ts">
  import SourceToggle from "./SourceToggle.svelte";
</script>

<section class="flex flex-col gap-4">
  <h2 class="text-lg font-medium">Battery</h2>
  <SourceToggle
    source="battery"
    label="Show battery status"
    hint="Charging animation when plugged in and an alert at 15%, 10% and 5%."
  />
</section>
