import type { HitRect } from "./contracts";
import { setHitRegions } from "./ipc";

function toPhysicalRect(element: HTMLElement): HitRect {
  const box = element.getBoundingClientRect();
  const scale = window.devicePixelRatio;
  return {
    x: box.left * scale,
    y: box.top * scale,
    width: box.width * scale,
    height: box.height * scale,
  };
}

export function trackHitRegions(elements: HTMLElement[]): () => void {
  let lastPublished = "";
  let runningTransitions = 0;
  let frame = 0;

  const publish = () => {
    const serialized = JSON.stringify(elements.map(toPhysicalRect));
    if (serialized === lastPublished) return;
    lastPublished = serialized;
    void setHitRegions(JSON.parse(serialized) as HitRect[]);
  };

  const animate = () => {
    publish();
    frame = runningTransitions > 0 ? requestAnimationFrame(animate) : 0;
  };

  const startAnimating = () => {
    runningTransitions += 1;
    if (!frame) frame = requestAnimationFrame(animate);
  };

  const stopAnimating = () => {
    runningTransitions = Math.max(0, runningTransitions - 1);
    publish();
  };

  const observer = new ResizeObserver(publish);
  elements.forEach((element) => {
    observer.observe(element);
    element.addEventListener("transitionrun", startAnimating);
    element.addEventListener("transitionend", stopAnimating);
    element.addEventListener("transitioncancel", stopAnimating);
  });
  window.addEventListener("resize", publish);
  publish();

  return () => {
    cancelAnimationFrame(frame);
    observer.disconnect();
    window.removeEventListener("resize", publish);
    elements.forEach((element) => {
      element.removeEventListener("transitionrun", startAnimating);
      element.removeEventListener("transitionend", stopAnimating);
      element.removeEventListener("transitioncancel", stopAnimating);
    });
  };
}
