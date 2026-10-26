import { onStream } from "./ipc";

export function useStream<T>(source: string, initial: T) {
  let current = $state<T>(initial);

  $effect(() => {
    const unlisten = onStream<T>(source, (payload) => (current = payload));
    return () => void unlisten.then((stop) => stop());
  });

  return {
    get value() {
      return current;
    },
  };
}
