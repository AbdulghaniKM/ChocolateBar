<script lang="ts">
  let { percent, muted = false, tall = false }: { percent: number; muted?: boolean; tall?: boolean } = $props();
</script>

<div
  class="overflow-hidden rounded-pill bg-white/10 {tall ? 'h-[0.625rem]' : 'h-[0.3125rem]'}"
  role="progressbar"
  aria-valuemin={0}
  aria-valuemax={100}
  aria-valuenow={percent}
>
  <div
    class="h-full rounded-pill transition-[width,background-color] duration-150 ease-island {muted ? 'bg-island-muted' : 'bg-island-text'}"
    style:width="{percent}%"
  ></div>
</div>
