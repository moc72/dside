<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Icon from "../Icon.vue";
import FileTree from "../FileTree.vue";
import MonacoEditor from "../MonacoEditor.vue";
import PdfView from "./PdfView.vue";
import { t } from "../../i18n";
import type { FileNode } from "../../types";
import { languageFromPath } from "../../lib/language";
import { mimeFor, previewKindFor, renderMarkdown } from "../../lib/preview";
import {
  initWorkspace,
  readFile,
  tree,
  treeError,
  treeLoading,
} from "../../composables/useWorkspace";
import { openFile } from "../../stores/workArea";
import { resolvedTheme } from "../../stores/settings";

const props = defineProps<{ path?: string | null }>();

const content = ref("");
const loading = ref(false);
const error = ref<string | null>(null);
const treeVisible = ref(true);

const selectedPath = computed(() => props.path ?? null);
const language = computed(() => (props.path ? languageFromPath(props.path) : "plaintext"));
const editorTheme = computed(() => (resolvedTheme.value === "light" ? "vs" : "vs-dark"));

const kind = computed(() => (props.path ? previewKindFor(props.path) : "text"));
const markdownMode = ref<"preview" | "source">("preview");
const imageSrc = ref("");

/**
 * Images come through Rust as base64 rather than through the asset protocol: the webview
 * blocks `asset://` requests from the dev-server origin ("protocols must match").
 */
watch(
  [() => props.path, kind],
  async ([path, currentKind]) => {
    markdownMode.value = "preview";
    imageSrc.value = "";
    if (!path || currentKind !== "image") return;
    error.value = null;
    try {
      const data = await invoke<string>("read_file_base64", { path });
      imageSrc.value = `data:${mimeFor(path)};base64,${data}`;
    } catch (cause) {
      error.value = String(cause);
    }
  },
  { immediate: true },
);

const rendered = computed(() =>
  kind.value === "markdown" ? renderMarkdown(content.value) : "",
);

const viewerOptions = {
  readOnly: true,
  minimap: { enabled: false },
  fontSize: 13,
  scrollBeyondLastLine: false,
};

async function load(path: string) {
  loading.value = true;
  error.value = null;
  try {
    // Binary previews bring their own loader. Trying to read a PNG as UTF-8 text fails,
    // and that failure used to surface as "could not read the file" over a preview that
    // was loading fine.
    const preview = previewKindFor(path);
    if (preview === "image" || preview === "pdf") {
      content.value = "";
      return;
    }
    content.value = await readFile(path);
  } catch (cause) {
    error.value = String(cause);
    content.value = "";
  } finally {
    loading.value = false;
  }
}

watch(
  () => props.path,
  (value) => {
    if (value) void load(value);
  },
  { immediate: true },
);

onMounted(() => {
  if (tree.value.length === 0) void initWorkspace();
});

function select(node: FileNode) {
  openFile(node.path);
}

function toggle(node: FileNode) {
  node.expanded = !node.expanded;
}
</script>

<template>
  <div class="viewfile" :class="{ 'no-tree': !treeVisible }">
    <div class="viewer">
      <template v-if="selectedPath && !error">
        <div v-if="kind !== 'text'" class="viewer-bar">
          <template v-if="kind === 'markdown'">
            <button
              class="mode"
              :class="{ active: markdownMode === 'preview' }"
              type="button"
              @click="markdownMode = 'preview'"
            >
              {{ t("viewFile.preview") }}
            </button>
            <button
              class="mode"
              :class="{ active: markdownMode === 'source' }"
              type="button"
              @click="markdownMode = 'source'"
            >
              {{ t("viewFile.source") }}
            </button>
          </template>
          <span class="path">{{ selectedPath }}</span>
        </div>

        <img v-if="kind === 'image'" class="image" :src="imageSrc" :alt="selectedPath ?? ''" />

        <PdfView v-else-if="kind === 'pdf'" :path="selectedPath" />

        <div
          v-else-if="kind === 'markdown' && markdownMode === 'preview'"
          class="markdown"
          v-html="rendered"
        ></div>

        <MonacoEditor
          v-else
          :path="selectedPath"
          :model-value="content"
          :language="language"
          :theme="editorTheme"
          :options="viewerOptions"
        />
      </template>

      <div v-else class="placeholder">
        <Icon name="doc" :size="28" />
        <p>
          {{ error ? t("viewFile.error") : loading ? t("viewFile.loading") : t("viewFile.hint") }}
        </p>
      </div>
    </div>

    <div v-if="treeVisible" class="tree-pane">
      <div class="tree-head">
        <span>{{ t("viewFile.workspace") }}</span>
        <button class="mini" type="button" :title="t('viewFile.hideTree')" @click="treeVisible = false">
          <Icon name="x" :size="12" />
        </button>
      </div>
      <div class="tree-body">
        <p v-if="treeLoading" class="note">{{ t("viewFile.loading") }}</p>
        <p v-else-if="treeError" class="note">{{ t("viewFile.error") }}</p>
        <FileTree
          v-else
          :nodes="tree"
          :selected-path="selectedPath"
          @select="select"
          @toggle="toggle"
        />
      </div>
    </div>

    <button
      v-else
      class="show-tree"
      type="button"
      :title="t('viewFile.showTree')"
      @click="treeVisible = true"
    >
      <Icon name="panel" :size="14" />
    </button>
  </div>
</template>

<style scoped>
.viewfile {
  position: relative;
  display: grid;
  grid-template-columns: minmax(0, 1fr) 220px;
  grid-template-rows: minmax(0, 1fr);
  height: 100%;
  min-height: 0;
}

.viewfile.no-tree {
  grid-template-columns: minmax(0, 1fr);
}

.viewer {
  min-width: 0;
  min-height: 0;
  position: relative;
  background: var(--bg);
  display: flex;
  flex-direction: column;
}

.viewer > .monaco-editor {
  flex: 1;
  min-height: 0;
}

.viewer-bar {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-panel);
}

.mode {
  height: 22px;
  padding: 0 10px;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-muted);
  font-size: 11px;
}

.mode.active {
  border-color: var(--border-strong);
  background: var(--bg-active);
  color: var(--text-strong);
}

.path {
  margin-left: auto;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text-muted);
  font-size: 11px;
}

.image {
  flex: 1;
  min-height: 0;
  object-fit: contain;
  padding: 16px;
  background: var(--bg);
}

.markdown {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 24px 28px;
  color: var(--text);
  font-size: 13.5px;
  line-height: 1.65;
}

.markdown :deep(h1),
.markdown :deep(h2),
.markdown :deep(h3) {
  margin: 1.4em 0 0.5em;
  color: var(--text-strong);
  line-height: 1.3;
}

.markdown :deep(h1) {
  padding-bottom: 0.3em;
  border-bottom: 1px solid var(--border);
  font-size: 1.6em;
}

.markdown :deep(h2) {
  font-size: 1.3em;
}

.markdown :deep(h3) {
  font-size: 1.1em;
}

.markdown :deep(p) {
  margin: 0.7em 0;
}

.markdown :deep(a) {
  color: var(--accent);
}

.markdown :deep(code) {
  padding: 0.15em 0.35em;
  border-radius: 3px;
  background: var(--bg-elevated);
  font-family: var(--mono);
  font-size: 0.92em;
}

.markdown :deep(pre) {
  padding: 12px 14px;
  border-radius: var(--radius);
  background: var(--bg-elevated);
  overflow-x: auto;
}

.markdown :deep(pre code) {
  padding: 0;
  background: none;
}

.markdown :deep(blockquote) {
  margin: 0.8em 0;
  padding: 0.2em 1em;
  border-left: 3px solid var(--border-strong);
  color: var(--text-muted);
}

.markdown :deep(ul),
.markdown :deep(ol) {
  padding-left: 1.5em;
}

.markdown :deep(table) {
  margin: 1em 0;
  border-collapse: collapse;
}

.markdown :deep(th),
.markdown :deep(td) {
  padding: 6px 10px;
  border: 1px solid var(--border);
}

.markdown :deep(img) {
  max-width: 100%;
}

.markdown :deep(hr) {
  margin: 1.5em 0;
  border: none;
  border-top: 1px solid var(--border);
}

.placeholder {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  color: var(--text-muted);
  text-align: center;
}

.tree-pane {
  display: flex;
  flex-direction: column;
  min-width: 0;
  border-left: 1px solid var(--border);
  background: var(--bg-elevated);
}

.tree-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px;
  border-bottom: 1px solid var(--border);
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.5px;
  text-transform: uppercase;
}

.mini {
  display: grid;
  place-items: center;
  width: 18px;
  height: 18px;
  border: none;
  border-radius: 3px;
  background: transparent;
  color: var(--text-muted);
}

.mini:hover {
  background: var(--bg-hover);
  color: var(--text-strong);
}

.tree-body {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 6px 0;
}

.note {
  padding: 4px 10px;
  margin: 0;
  color: var(--text-muted);
  font-size: 12px;
}

.show-tree {
  position: absolute;
  top: 8px;
  right: 8px;
  display: grid;
  place-items: center;
  width: 26px;
  height: 26px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-panel);
  color: var(--text-muted);
}

.show-tree:hover {
  color: var(--text-strong);
}
</style>
