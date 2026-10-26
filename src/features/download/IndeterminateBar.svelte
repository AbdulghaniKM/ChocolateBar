<script lang="ts">
  let { done = false, tall = false }: { done?: boolean; tall?: boolean } = $props();
</script>

<div
  class="relative w-full overflow-hidden rounded-pill bg-white/10 {tall ? 'h-[0.375rem]' : 'h-[0.1875rem]'}"
  role="progressbar"
  aria-busy={!done}
>
  <div class="track absolute inset-y-0 left-0 w-full rounded-pill" class:done></div>
</div>

<style>
  .track {
    width: 40%;
    background: linear-gradient(90deg, transparent, #38bdf8 40%, #7dd3fc 60%, transparent);
    animation: slide 1.3s cubic-bezier(0.45, 0, 0.25, 1) infinite;
  }
  .track.done {
    width: 100%;
    background: #22c55e;
    animation: none;
    transform: none;
  }
  @keyframes slide {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(250%);
    }
  }
</style>
