import { computed, ref, watch } from "vue";
import { saveUiState } from "../lib/uiState";
import { workAuto } from "./ui";
import type { WorkAreaTab, WorkAreaTabKind } from "../types";

const PROGRESS_TAB: WorkAreaTab = { id: "progress", kind: "progress" };

/** Where the work area lived before it moved into the settings table. */
const LEGACY_KEY = "ds-ide.workarea";
const UI_KEY = "workarea";

const KINDS: WorkAreaTabKind[] = ["progress", "file", "skills", "schedules", "terminal", "git", "logs"];

export const tabs = ref<WorkAreaTab[]>([{ ...PROGRESS_TAB }]);

export const activeTabId = ref<string>(PROGRESS_TAB.id);

export const visible = ref(true);

/**
 * The file tree is part of the file tab, so it survives switching tabs and the companion
 * opening a file. Remembered with the rest of the work area — a tree the user put away
 * should not be back after a restart or an edit by the partner.
 */
export const fileTreeVisible = ref(true);

export const activeTab = computed(
  () => tabs.value.find((tab) => tab.id === activeTabId.value) ?? null,
);

/**
 * Bumped when a file the companion just wrote comes back. The file tab watches it and reads
 * the file again — the path alone does not change, so its own watcher would stay silent.
 */
export const fileRevision = ref(0);

/** The tab that recently came into view on its own, so the bar can point at it briefly. */
export const attentionTabId = ref("");

let attentionTimer: number | undefined;

export function pulseTab(id: string): void {
  attentionTabId.value = id;
  if (attentionTimer) window.clearTimeout(attentionTimer);
  attentionTimer = window.setTimeout(() => {
    attentionTimer = undefined;
    attentionTabId.value = "";
  }, 1800);
}

/** Reloads the preview only when that exact file is already the open file tab. */
export function refreshFile(path: string): void {
  const tab = tabs.value.find((entry) => entry.kind === "file" && entry.filePath === path);
  if (tab) fileRevision.value += 1;
}

export function activateTab(id: string): void {
  if (tabs.value.some((tab) => tab.id === id)) activeTabId.value = id;
}

export function showWorkArea(): void {
  // An explicit request wins over the narrow-window rule; the next resize re-evaluates it.
  workAuto.value = false;
  visible.value = true;
}

export function toggleWorkArea(): void {
  if (workAuto.value) {
    workAuto.value = false;
    visible.value = true;
    return;
  }
  visible.value = !visible.value;
}

export function openTab(kind: WorkAreaTabKind, filePath?: string): string {
  const id = kind === "file" && filePath ? `file:${filePath}` : kind;
  const existing = tabs.value.find((tab) => tab.id === id);
  if (!existing) {
    tabs.value.push({ id, kind, filePath: filePath ?? null });
  }
  showWorkArea();
  activeTabId.value = id;
  return id;
}

export function openFile(path: string): string {
  const id = "file";
  const existing = tabs.value.find((tab) => tab.id === id);
  if (existing) existing.filePath = path;
  else tabs.value.push({ id, kind: "file", filePath: path });
  showWorkArea();
  activeTabId.value = id;
  return id;
}

/**
 * Opens a file and asks the tab to show its snapshot history. The tool row uses this after
 * a write/edit; the tab watches the counter so a second click on the same path still acts.
 */
export const historyRequest = ref(0);
export const historyPath = ref<string | null>(null);

export function openFileHistory(path: string): void {
  openFile(path);
  historyPath.value = path;
  historyRequest.value += 1;
}

export function closeTab(id: string): void {
  const index = tabs.value.findIndex((tab) => tab.id === id);
  if (index === -1) return;
  tabs.value.splice(index, 1);
  if (activeTabId.value !== id) return;
  const next = tabs.value[index] ?? tabs.value[index - 1] ?? null;
  activeTabId.value = next ? next.id : "";
}

// ---------------------------------------------------------------------------
// Restoring the work area
// ---------------------------------------------------------------------------

interface StoredWorkArea {
  visible?: boolean;
  activeTabId?: string;
  tabs?: unknown;
  treeVisible?: boolean;
}

function isTab(value: unknown): value is WorkAreaTab {
  if (!value || typeof value !== "object") return false;
  const tab = value as WorkAreaTab;
  if (typeof tab.id !== "string" || !KINDS.includes(tab.kind)) return false;
  return tab.filePath === undefined || tab.filePath === null || typeof tab.filePath === "string";
}

/** Reads and drops the pre-database value, so it is imported at most once. */
function takeLegacyVisible(): boolean | null {
  try {
    const raw = localStorage.getItem(LEGACY_KEY);
    if (raw === null) return null;
    localStorage.removeItem(LEGACY_KEY);
    const parsed = JSON.parse(raw) as { visible?: boolean };
    return typeof parsed.visible === "boolean" ? parsed.visible : null;
  } catch {
    return null;
  }
}

/**
 * Brings back the open tabs, the active one — and with a `file` tab, the file that was last
 * previewed. Tabs that no longer make sense are dropped instead of failing the whole restore.
 */
export function initWorkArea(stored: unknown): void {
  // Armed before the restore, so the repaired state is written back straight away: a stale
  // tab id, or a visibility imported from the old localStorage key, then survives the next
  // launch even if nothing is clicked in between.
  watch(
    [tabs, activeTabId, visible, fileTreeVisible],
    () =>
      saveUiState(UI_KEY, {
        visible: visible.value,
        activeTabId: activeTabId.value,
        tabs: tabs.value,
        treeVisible: fileTreeVisible.value,
      }),
    { deep: true },
  );

  const saved = (stored ?? {}) as StoredWorkArea;

  const restored: WorkAreaTab[] = [{ ...PROGRESS_TAB }];
  if (Array.isArray(saved.tabs)) {
    for (const tab of saved.tabs) {
      if (!isTab(tab)) continue;
      if (restored.some((known) => known.id === tab.id)) continue;
      restored.push(tab);
    }
  }
  tabs.value = restored;

  const wanted = saved.activeTabId;
  activeTabId.value = wanted && restored.some((tab) => tab.id === wanted) ? wanted : restored[0].id;

  if (typeof saved.visible === "boolean") visible.value = saved.visible;
  else {
    const legacy = takeLegacyVisible();
    if (legacy !== null) visible.value = legacy;
  }

  if (typeof saved.treeVisible === "boolean") fileTreeVisible.value = saved.treeVisible;
}
