import { onBeforeUnmount, onMounted } from "vue";
import { railAuto, railCollapsed, railWidth, workAuto, workWidth } from "../stores/ui";
import { visible as workVisible } from "../stores/workArea";

/**
 * How much room the conversation needs before a side panel is worth squeezing it. The two
 * values are hysteresis: hide below the first, show again only above the second, so a window
 * resting on a threshold does not flicker.
 */
const RAIL_HIDE_BELOW = 720;
const RAIL_SHOW_ABOVE = 780;
const WORK_HIDE_BELOW = 640;
const WORK_SHOW_ABOVE = 700;

/** The stage keeps 10 px on each side; every neighbour costs a 10 px gap. */
const STAGE_PADDING = 20;
const GAP = 10;

function apply(width: number): void {
  const body = Math.max(0, width - STAGE_PADDING);
  const rail = (body * railWidth.value) / 100 + GAP;
  const work = (body * workWidth.value) / 100 + GAP;

  // The rail goes first: navigation matters, but the conversation is what must stay readable.
  if (!railCollapsed.value) {
    const center = body - rail - (workVisible.value ? work : 0);
    railAuto.value = center < (railAuto.value ? RAIL_SHOW_ABOVE : RAIL_HIDE_BELOW);
  } else {
    railAuto.value = false;
  }

  // Then the work area, measured with the rail as it will actually be shown.
  if (workVisible.value) {
    const center = body - work - (railCollapsed.value || railAuto.value ? 0 : rail);
    workAuto.value = center < (workAuto.value ? WORK_SHOW_ABOVE : WORK_HIDE_BELOW);
  } else {
    workAuto.value = false;
  }
}

/** Installs the window watcher; call once from the main window component. */
export function useResponsiveLayout(): void {
  const onResize = () => apply(window.innerWidth);
  onMounted(() => {
    apply(window.innerWidth);
    window.addEventListener("resize", onResize);
  });
  onBeforeUnmount(() => window.removeEventListener("resize", onResize));
}
