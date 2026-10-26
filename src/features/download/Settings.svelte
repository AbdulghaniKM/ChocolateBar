<script lang="ts">
  import SourceToggle from "../battery/SourceToggle.svelte";
</script>

<section class="flex flex-col gap-4">
  <h2 class="text-lg font-medium">Downloads</h2>
  <SourceToggle
    source="download"
    label="Show download progress"
    hint="Watches your Downloads folder for .crdownload, .part and .partial files."
  />
</section>
