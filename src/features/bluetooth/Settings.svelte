<script lang="ts">
  import SourceToggle from "../battery/SourceToggle.svelte";
</script>

<section class="flex flex-col gap-4">
  <h2 class="text-lg font-medium">Bluetooth</h2>
  <SourceToggle
    source="bluetooth"
    label="Show Bluetooth devices"
    hint="Battery level for paired devices plus connect, disconnect and low battery alerts."
  />
</section>
