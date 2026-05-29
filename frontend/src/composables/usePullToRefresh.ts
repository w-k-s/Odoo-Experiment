import { onBeforeUnmount, onMounted, type Ref } from "vue";

/**
 * Minimal touch-based pull-to-refresh. When the user drags down past
 * `threshold` px while the target is scrolled to the top, `onRefresh` fires.
 */
export function usePullToRefresh(
  target: Ref<HTMLElement | null>,
  onRefresh: () => Promise<void> | void,
  threshold = 70,
) {
  let startY = 0;
  let pulling = false;

  function onTouchStart(e: TouchEvent) {
    const el = target.value;
    if (!el || el.scrollTop > 0) return;
    startY = e.touches[0].clientY;
    pulling = true;
  }

  function onTouchMove(e: TouchEvent) {
    if (pulling && e.touches[0].clientY - startY < 0) pulling = false;
  }

  async function onTouchEnd(e: TouchEvent) {
    if (!pulling) return;
    const distance = e.changedTouches[0].clientY - startY;
    pulling = false;
    if (distance > threshold) await onRefresh();
  }

  onMounted(() => {
    const el = target.value;
    if (!el) return;
    el.addEventListener("touchstart", onTouchStart, { passive: true });
    el.addEventListener("touchmove", onTouchMove, { passive: true });
    el.addEventListener("touchend", onTouchEnd, { passive: true });
  });

  onBeforeUnmount(() => {
    const el = target.value;
    if (!el) return;
    el.removeEventListener("touchstart", onTouchStart);
    el.removeEventListener("touchmove", onTouchMove);
    el.removeEventListener("touchend", onTouchEnd);
  });
}
