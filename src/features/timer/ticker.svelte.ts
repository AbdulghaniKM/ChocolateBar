const DEFAULT_INTERVAL_MS = 100;

export function useNow(intervalMs = DEFAULT_INTERVAL_MS) {
  let now = $state(Date.now());

  $effect(() => {
    const timer = setInterval(() => (now = Date.now()), intervalMs);
    return () => clearInterval(timer);
  });

  return {
    get value() {
      return now;
    },
  };
}
