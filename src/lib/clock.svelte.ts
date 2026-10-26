const timeFormat = new Intl.DateTimeFormat([], { hour: "2-digit", minute: "2-digit" });
const dateFormat = new Intl.DateTimeFormat([], { weekday: "long", day: "numeric", month: "long" });
const shortDateFormat = new Intl.DateTimeFormat([], { weekday: "short", day: "numeric" });

export function useClock() {
  let now = $state(new Date());

  $effect(() => {
    const timer = setInterval(() => (now = new Date()), 1000);
    return () => clearInterval(timer);
  });

  return {
    get time() {
      return timeFormat.format(now);
    },
    get date() {
      return dateFormat.format(now);
    },
    get shortDate() {
      return shortDateFormat.format(now);
    },
  };
}
