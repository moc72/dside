import { ref, watch } from "vue";
import type { AppView } from "../types";
import { loadJson, saveJson } from "../lib/persist";

export const RAIL_MIN = 12;
export const RAIL_MAX = 28;
export const WORK_MIN = 15;
export const WORK_MAX = 55;
export const CENTER_MIN = 30;

const STORAGE_KEY = "ds-ide.ui.v2";

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

interface StoredUi {
  railWidth: number;
  workWidth: number;
  railCollapsed: boolean;
}

const stored = loadJson<Partial<StoredUi>>(STORAGE_KEY, {});

export const view = ref<AppView>("chat");
export const railCollapsed = ref(stored.railCollapsed ?? false);
export const railWidth = ref(clamp(stored.railWidth ?? 15, RAIL_MIN, RAIL_MAX));
// A file tab reserves 220 px for the tree; at 15 % the preview was squeezed to a
// sliver on a 1920-wide screen. 30 % leaves room for both without crowding the chat.
// 15 % was the old default, so an untouched value is lifted to the new one; a width
// the user actually chose stays.
const storedWork = stored.workWidth;
const workStart = storedWork === undefined || storedWork === 15 ? 30 : storedWork;
export const workWidth = ref(clamp(workStart, WORK_MIN, WORK_MAX));

/**
 * Set by the responsive layout, never by the user and never persisted: the panel is hidden
 * because the window is too narrow, not because it was put away. Manual toggles clear the
 * flag, so a forced-open panel stays open until the window crosses a threshold again.
 */
export const railAuto = ref(false);
export const workAuto = ref(false);


watch([railWidth, workWidth, railCollapsed], () => {
  saveJson(STORAGE_KEY, {
    railWidth: railWidth.value,
    workWidth: workWidth.value,
    railCollapsed: railCollapsed.value,
  });
});

/** Which settings page to open next — set before `openSettings()` when linking deep. */
export const settingsItem = ref<string | null>(null);

export function openSettings(item?: string): void {
  settingsItem.value = item ?? null;
  view.value = "settings";
}

export function closeSettings(): void {
  view.value = "chat";
}

export function toggleRail(): void {
  // An auto-hidden rail is shown on the first click instead of flipping the manual state.
  if (railAuto.value) {
    railAuto.value = false;
    railCollapsed.value = false;
    return;
  }
  railCollapsed.value = !railCollapsed.value;
}

export const searchOpen = ref(false);

/** The web design panel: URL row plus the list of browser windows we own. */
export const webOpen = ref(false);

export function toggleWeb(): void {
  webOpen.value = !webOpen.value;
}

export function openSearch(): void {
  searchOpen.value = true;
}

export function closeSearch(): void {
  searchOpen.value = false;
}

export function setRailWidth(value: number): void {
  railWidth.value = clamp(value, RAIL_MIN, RAIL_MAX);
}

export function setWorkWidth(value: number): void {
  workWidth.value = clamp(value, WORK_MIN, WORK_MAX);
}
