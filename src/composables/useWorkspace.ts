import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { FileNode } from "../types";
import { settings } from "../stores/settings";

export const tree = ref<FileNode[]>([]);
export const treeLoading = ref(false);
export const treeError = ref<string | null>(null);

export async function reloadTree(): Promise<void> {
  if (!settings.workspace) {
    tree.value = [];
    return;
  }
  treeLoading.value = true;
  treeError.value = null;
  try {
    tree.value = await invoke<FileNode[]>("read_workspace_tree", {
      root: settings.workspace,
      depth: 2,
    });
  } catch (error) {
    treeError.value = String(error);
    tree.value = [];
  } finally {
    treeLoading.value = false;
  }
}

export async function ensureWorkspace(): Promise<void> {
  if (settings.workspace) return;
  try {
    settings.workspace = await invoke<string>("default_workspace");
  } catch {
    settings.workspace = "";
  }
}

export async function initWorkspace(): Promise<void> {
  await ensureWorkspace();
  await reloadTree();
}

export async function readFile(path: string): Promise<string> {
  return invoke<string>("read_text_file", { path });
}
