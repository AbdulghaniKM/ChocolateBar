<script lang="ts">
  import SourceToggle from "../battery/SourceToggle.svelte";
</script>

<section class="flex flex-col gap-4">
  <h2 class="text-lg font-medium">Keyboard</h2>
  <SourceToggle
    source="keyboard"
    label="Show layout and Caps Lock indicator"
    hint="Flashes the active input language (AR, EN, ...) and Caps Lock changes."
  />
</section>
