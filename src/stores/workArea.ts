import { computed, ref, watch } from "vue";
import { loadJson, saveJson } from "../lib/persist";
import type { WorkAreaTab, WorkAreaTabKind } from "../types";

const PROGRESS_TAB: WorkAreaTab = { id: "progress", kind: "progress" };

const STORAGE_KEY = "ds-ide.workarea";

export const tabs = ref<WorkAreaTab[]>([{ ...PROGRESS_TAB }]);

export const activeTabId = ref<string>(PROGRESS_TAB.id);

export const visible = ref(loadJson<{ visible: boolean }>(STORAGE_KEY, { visible: true }).visible);

watch(visible, (value) => saveJson(STORAGE_KEY, { visible: value }));

export const activeTab = computed(
  () => tabs.value.find((tab) => tab.id === activeTabId.value) ?? null,
);

export function activateTab(id: string): void {
  if (tabs.value.some((tab) => tab.id === id)) activeTabId.value = id;
}

export function showWorkArea(): void {
  visible.value = true;
}

export function toggleWorkArea(): void {
  visible.value = !visible.value;
}

export function openTab(kind: WorkAreaTabKind, filePath?: string): string {
  const id = kind === "file" && filePath ? `file:${filePath}` : kind;
  const existing = tabs.value.find((tab) => tab.id === id);
  if (!existing) {
    tabs.value.push({ id, kind, filePath: filePath ?? null });
  }
  visible.value = true;
  activeTabId.value = id;
  return id;
}

export function openFile(path: string): string {
  const id = "file";
  const existing = tabs.value.find((tab) => tab.id === id);
  if (existing) existing.filePath = path;
  else tabs.value.push({ id, kind: "file", filePath: path });
  visible.value = true;
  activeTabId.value = id;
  return id;
}

export function closeTab(id: string): void {
  const index = tabs.value.findIndex((tab) => tab.id === id);
  if (index === -1) return;
  tabs.value.splice(index, 1);
  if (activeTabId.value !== id) return;
  const next = tabs.value[index] ?? tabs.value[index - 1] ?? null;
  activeTabId.value = next ? next.id : "";
}
