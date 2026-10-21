import { computed, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { EditorCandidate } from "../types";
import { settings } from "../stores/settings";
import { ensureWorkspace } from "./useWorkspace";

export const editors = ref<EditorCandidate[]>([]);
export const detecting = ref(false);
export const detectError = ref<string | null>(null);

export async function detectEditors(): Promise<void> {
  detecting.value = true;
  detectError.value = null;
  try {
    const found = await invoke<EditorCandidate[]>("detect_editors");
    editors.value = found;
    const pinned = settings.pinnedEditorId;
    if (!pinned || !found.some((editor) => editor.id === pinned)) {
      settings.pinnedEditorId = found.length > 0 ? found[0].id : null;
    }
  } catch (error) {
    detectError.value = String(error);
    editors.value = [];
  } finally {
    detecting.value = false;
  }
}

export const activeEditor = computed<EditorCandidate | null>(
  () => editors.value.find((editor) => editor.id === settings.pinnedEditorId) ?? null,
);

export async function openInEditor(): Promise<void> {
  const editor = activeEditor.value;
  if (!editor) return;
  // A first click may happen before the file tree has ever resolved the workspace.
  await ensureWorkspace();
  await invoke("open_workspace_in_editor", {
    editorId: editor.id,
    workspace: settings.workspace,
  });
}

export function useEditors(): void {
  onMounted(() => {
    if (editors.value.length === 0) void detectEditors();
  });
}
