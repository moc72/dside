<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Icon from "../Icon.vue";
import FileTree from "../FileTree.vue";
import MonacoEditor from "../MonacoEditor.vue";
import MonacoDiff from "../MonacoDiff.vue";
import { everydayMode } from "../../engine/mode";
import PdfView from "./PdfView.vue";
import { t } from "../../i18n";
import type { FileNode, FileVersionInfo } from "../../types";
import { languageFromPath } from "../../lib/language";
import { escapeHtml, mimeFor, previewKindFor, renderMarkdown, rtfToText } from "../../lib/preview";
import { renderDocx, renderSheet, renderSlides } from "../../lib/office";
import { renderLegacySlides, renderLegacyWord } from "../../lib/binaryOffice";
import { renderOdfSlides, renderOdfText } from "../../lib/odf";
import {
  initWorkspace,
  loadChildren,
  readFile,
  tree,
  treeError,
  treeLoading,
} from "../../composables/useWorkspace";
import {
  fileRevision,
  fileTreeVisible,
  historyPath,
  historyRequest,
  openFile,
} from "../../stores/workArea";
import { activeSessionId } from "../../stores/sessions";
import { resolvedTheme } from "../../stores/settings";

const props = defineProps<{ path?: string | null }>();

const content = ref("");
const loading = ref(false);
const error = ref<string | null>(null);

const selectedPath = computed(() => props.path ?? null);
const language = computed(() => (props.path ? languageFromPath(props.path) : "plaintext"));
const editorTheme = computed(() => (resolvedTheme.value === "light" ? "vs" : "vs-dark"));

const kind = computed(() => (props.path ? previewKindFor(props.path) : "text"));
const markdownMode = ref<"preview" | "source">("preview");

/** The snapshot history: the list, the selected version and the diff against the file. */
const historyOpen = ref(false);
const historyLoading = ref(false);
const historyError = ref<string | null>(null);
const versions = ref<FileVersionInfo[]>([]);
const selectedVersionId = ref<number | null>(null);
const originalContent = ref<string | null>(null);
const restoring = ref(false);
const historyAvailable = computed(() => kind.value === "text" || kind.value === "markdown");

watch(
  () => props.path,
  () => {
    markdownMode.value = "preview";
    historyOpen.value = false;
    historyError.value = null;
    versions.value = [];
    selectedVersionId.value = null;
    originalContent.value = null;
  },
);

/** Spreadsheets: the value a cell shows, or the formula that produced it. */
const sheetView = ref<"values" | "formulas">("values");
/** The file as it was read, so switching the view does not read it again. */
const officeSource = ref<string | null>(null);
const imageSrc = ref("");

/**
 * Images come through Rust as base64 rather than through the asset protocol: the webview
 * blocks `asset://` requests from the dev-server origin ("protocols must match").
 */
watch(sheetView, () => {
  if (kind.value === "sheet" && officeSource.value) {
    officeHtml.value = renderSheet(officeSource.value, sheetView.value);
  }
});

watch(
  [() => props.path, kind, fileRevision],
  async ([path, currentKind]) => {
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

const officeHtml = ref("");

/** The office family: each format has its own reader, none of them is plain text. */
const OFFICE_KINDS = ["docx", "doc", "odt", "sheet", "rtf", "slides", "ppt", "odp"];

watch(sheetView, () => {
  if (kind.value === "sheet" && officeSource.value) {
    officeHtml.value = renderSheet(officeSource.value, sheetView.value);
  }
});

watch(
  [() => props.path, kind, fileRevision],
  async ([path, currentKind]) => {
    officeHtml.value = "";
    if (!path) return;
    if (!OFFICE_KINDS.includes(currentKind)) return;
    error.value = null;
    try {
      const base64 = await invoke<string>("read_file_base64", { path });
      officeSource.value = currentKind === "sheet" ? base64 : null;
      if (currentKind === "docx") officeHtml.value = await renderDocx(base64);
      else if (currentKind === "doc") officeHtml.value = await renderLegacyWord(base64);
      else if (currentKind === "odt") officeHtml.value = await renderOdfText(base64);
      else if (currentKind === "sheet") officeHtml.value = renderSheet(base64, sheetView.value);
      else if (currentKind === "slides") officeHtml.value = await renderSlides(base64);
      else if (currentKind === "ppt") officeHtml.value = renderLegacySlides(base64);
      else if (currentKind === "odp") officeHtml.value = await renderOdfSlides(base64);
      else officeHtml.value = `<pre>${escapeHtml(rtfToText(atob(base64)))}</pre>`;
    } catch (cause) {
      error.value = String(cause);
    }
  },
  { immediate: true },
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
    if (preview !== "text" && preview !== "markdown") {
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

function originLabel(origin: string): string {
  if (origin === "write") return t("viewFile.historyWrite");
  if (origin === "edit") return t("viewFile.historyEdit");
  return t("viewFile.historyRevert");
}

function formatVersionTime(seconds: number): string {
  return new Date(seconds * 1000).toLocaleString();
}

async function loadVersion(id: number): Promise<void> {
  const body = await invoke<string | null>("file_version_content", { id });
  originalContent.value = body ?? "";
}

async function loadVersions(): Promise<void> {
  const path = selectedPath.value;
  if (!path || !historyAvailable.value) return;
  historyLoading.value = true;
  historyError.value = null;
  try {
    versions.value = await invoke<FileVersionInfo[]>("file_versions", { path });
    const stillThere = versions.value.some((version) => version.id === selectedVersionId.value);
    if (!stillThere) selectedVersionId.value = versions.value[0]?.id ?? null;
    if (selectedVersionId.value) await loadVersion(selectedVersionId.value);
    else originalContent.value = null;
  } catch (error) {
    historyError.value = `${t("viewFile.historyError")} ${String(error)}`;
  } finally {
    historyLoading.value = false;
  }
}

async function openHistory(): Promise<void> {
  if (!historyAvailable.value) return;
  historyOpen.value = true;
  await loadVersions();
}

function toggleHistory(): void {
  if (historyOpen.value) {
    historyOpen.value = false;
    return;
  }
  void openHistory();
}

async function selectVersion(id: number): Promise<void> {
  selectedVersionId.value = id;
  historyError.value = null;
  try {
    await loadVersion(id);
  } catch (error) {
    historyError.value = `${t("viewFile.historyError")} ${String(error)}`;
  }
}

async function restoreSelected(): Promise<void> {
  const id = selectedVersionId.value;
  if (!id || restoring.value || !selectedPath.value) return;
  if (!window.confirm(t("viewFile.restoreConfirm"))) return;
  restoring.value = true;
  historyError.value = null;
  try {
    await invoke("file_revert", { id, sessionId: activeSessionId.value ?? "" });
    await load(selectedPath.value);
    historyOpen.value = false;
    versions.value = [];
    selectedVersionId.value = null;
    originalContent.value = null;
  } catch (error) {
    historyError.value = `${t("viewFile.restoreFailed")} ${String(error)}`;
  } finally {
    restoring.value = false;
  }
}

watch(
  [() => props.path, fileRevision],
  ([value]) => {
    if (value) void load(value);
    if (value && historyOpen.value) void loadVersions();
  },
  { immediate: true },
);

watch(historyRequest, async () => {
  const requestPath = historyPath.value;
  if (!requestPath) return;
  await nextTick();
  if (requestPath === selectedPath.value) await openHistory();
});

onMounted(() => {
  if (tree.value.length === 0) void initWorkspace();
});

/**
 * After a file is picked in the tree, give the eyes a moment to see where it went, then get
 * the tree out of the way. The collapse itself is a CSS transition; this is only the pause
 * before it starts. Choosing another file resets the pause, and using the tree cancels it.
 */
const AUTO_HIDE_MS = 2000;
let hideTimer: number | undefined;

function cancelTreeHide(): void {
  if (hideTimer !== undefined) {
    window.clearTimeout(hideTimer);
    hideTimer = undefined;
  }
}

function scheduleTreeHide(): void {
  cancelTreeHide();
  hideTimer = window.setTimeout(() => {
    hideTimer = undefined;
    fileTreeVisible.value = false;
  }, AUTO_HIDE_MS);
}

function toggleTree(): void {
  cancelTreeHide();
  fileTreeVisible.value = !fileTreeVisible.value;
}

function closeTree(): void {
  cancelTreeHide();
  fileTreeVisible.value = false;
}

onBeforeUnmount(cancelTreeHide);

function select(node: FileNode) {
  openFile(node.path);
  // The companion never goes through here: `openFile` on its behalf leaves the tree alone.
  scheduleTreeHide();
}

async function toggle(node: FileNode) {
  // Browsing counts as using the tree — do not pull it away while folders open.
  cancelTreeHide();
  node.expanded = !node.expanded;
  if (node.expanded && !node.loaded) await loadChildren(node);
}
</script>

<template>
  <div class="viewfile" :class="{ 'no-tree': !fileTreeVisible }">
    <div class="viewer">
      <template v-if="selectedPath && !error">
        <div class="viewer-bar">
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
          <button
            v-if="historyAvailable"
            class="mode"
            :class="{ active: historyOpen }"
            type="button"
            @click="toggleHistory"
          >
            {{ t("viewFile.history") }}
          </button>
          <!-- The same fixed line for every kind: markdown's mode switch, code's path, and
               the tree toggle. It does not scroll with Monaco, so the path stays visible. -->
          <span class="path">{{ selectedPath }}</span>
          <button
            class="mini tree-toggle"
            :class="{ active: fileTreeVisible }"
            type="button"
            :title="fileTreeVisible ? t('viewFile.hideTree') : t('viewFile.showTree')"
            :aria-pressed="fileTreeVisible"
            @click="toggleTree"
          >
            <Icon name="panel" :size="14" />
          </button>
        </div>

        <div v-if="historyOpen" class="history">
          <div class="history-list">
            <p v-if="historyLoading" class="note">{{ t("viewFile.loading") }}</p>
            <p v-else-if="historyError" class="note warn">{{ historyError }}</p>
            <p v-else-if="!versions.length" class="note">{{ t("viewFile.historyEmpty") }}</p>
            <button
              v-for="version in versions"
              :key="version.id"
              class="history-item"
              :class="{ active: version.id === selectedVersionId }"
              type="button"
              @click="selectVersion(version.id)"
            >
              <span class="history-time">{{ formatVersionTime(version.createdAt) }}</span>
              <span class="history-origin">
                {{ originLabel(version.origin) }}
                <span v-if="!version.existed" class="history-new">
                  · {{ t("viewFile.historyCreated") }}
                </span>
              </span>
            </button>
          </div>
          <div class="history-view">
            <div class="history-actions">
              <span class="history-hint">{{ t("viewFile.historyHint") }}</span>
              <button
                class="restore"
                type="button"
                :disabled="!selectedVersionId || restoring"
                @click="restoreSelected"
              >
                <Icon name="reload" :size="13" />
                <span>{{ t("viewFile.restore") }}</span>
              </button>
            </div>
            <div v-if="originalContent !== null" class="history-diff">
              <MonacoDiff
                :original="originalContent"
                :modified="content"
                :language="language"
                :theme="editorTheme"
              />
            </div>
            <p v-else class="note">{{ t("viewFile.historyHint") }}</p>
          </div>
        </div>

        <template v-else>
          <img v-if="kind === 'image'" class="image" :src="imageSrc" :alt="selectedPath ?? ''" />

        <PdfView v-else-if="kind === 'pdf'" :path="selectedPath" />

        <div
          v-else-if="kind === 'markdown' && markdownMode === 'preview'"
          class="markdown"
          v-html="rendered"
        ></div>

        <div
          v-else-if="OFFICE_KINDS.includes(kind)"
          class="markdown"
          v-html="officeHtml"
        ></div>

        <div v-else-if="kind === 'unsupported'" class="placeholder">
          <Icon name="doc" :size="28" />
          <p>{{ t("viewFile.unsupported") }}</p>
        </div>

        <!-- Everyday mode shows the text itself: no code editor, no syntax machinery. -->
        <pre v-else-if="everydayMode" class="plain">{{ content }}</pre>

        <MonacoEditor
          v-else
          :path="selectedPath"
          :model-value="content"
          :language="language"
          :theme="editorTheme"
          :options="viewerOptions"
        />
        </template>
      </template>

      <div v-else class="placeholder">
        <Icon name="doc" :size="28" />
        <p>
          {{ error ? t("viewFile.error") : loading ? t("viewFile.loading") : t("viewFile.hint") }}
        </p>
      </div>
    </div>

    <div
      class="tree-pane"
      :inert="!fileTreeVisible || undefined"
      :aria-hidden="fileTreeVisible ? undefined : 'true'"
    >
      <div class="tree-head">
        <span>{{ t("viewFile.workspace") }}</span>
        <button class="mini" type="button" :title="t('viewFile.hideTree')" @click="closeTree">
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
  /* The tree steps aside after a pick instead of vanishing: same track list, only the
     second column goes to zero, so the browser can interpolate it. */
  transition: grid-template-columns 260ms ease;
}

.viewfile.no-tree {
  grid-template-columns: minmax(0, 1fr) 0px;
}

.viewer {
  min-width: 0;
  min-height: 0;
  position: relative;
  background: var(--bg);
  display: flex;
  flex-direction: column;
}

.plain {
  margin: 0;
  padding: 14px 16px;
  overflow: auto;
  height: 100%;
  color: var(--text);
  font-family: var(--mono);
  font-size: 12px;
  line-height: 1.55;
  white-space: pre-wrap;
  word-break: break-word;
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
  flex: 1 1 auto;
  min-width: 0;
  margin-left: auto;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  text-align: right;
  color: var(--text-muted);
  font-size: 11px;
}

.tree-toggle {
  flex: none;
  width: 22px;
  height: 22px;
}

.tree-toggle.active {
  color: var(--accent);
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
  font-size: var(--reading-font-size);
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

/* Presentations: one card per slide. The text is real, the layout is not — so it reads as a
   list of slides instead of pretending to be the original page. */
.markdown :deep(.slide) {
  margin: 0 0 1.4em;
  padding: 14px 16px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg-elevated);
}

.markdown :deep(.slide h3) {
  margin: 0 0 0.7em;
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.5px;
  text-transform: uppercase;
}

.markdown :deep(.slide p) {
  margin: 0.3em 0;
}

.markdown :deep(.slide .slide-title) {
  color: var(--text-strong);
  font-weight: 600;
}

.markdown :deep(.sheet-name) {
  margin: 1.4em 0 0.5em;
  color: var(--text-strong);
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.2px;
}

.markdown :deep(.sheet-name:first-child) {
  margin-top: 0;
}

.markdown :deep(.truncated) {
  color: var(--text-muted);
  font-size: 0.92em;
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
  overflow: hidden;
  border-left: 1px solid var(--border);
  background: var(--bg-elevated);
  transition:
    border-color 260ms ease,
    opacity 260ms ease;
}

.viewfile.no-tree .tree-pane {
  border-left-color: transparent;
  opacity: 0;
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

.note.warn {
  color: var(--danger);
}

.history {
  flex: 1;
  min-height: 0;
  display: flex;
}

.history-list {
  width: 220px;
  min-width: 0;
  overflow-y: auto;
  padding: 6px;
  border-right: 1px solid var(--border);
  background: var(--bg-panel);
}

.history-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
  width: 100%;
  padding: 7px 8px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text);
  text-align: left;
}

.history-item:hover {
  background: var(--bg-hover);
}

.history-item.active {
  background: var(--bg-active);
  color: var(--text-strong);
}

.history-time {
  color: var(--text-muted);
  font-size: 11px;
}

.history-origin {
  font-size: 12px;
}

.history-new {
  color: var(--accent);
  font-size: 11px;
}

.history-view {
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.history-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 10px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-panel);
}

.history-hint {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text-muted);
  font-size: 11px;
}

.restore {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  flex: none;
  height: 26px;
  padding: 0 10px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-strong);
  font-size: 12px;
}

.restore:hover:not(:disabled) {
  background: var(--bg-hover);
}

.restore:disabled {
  opacity: 0.5;
}

.history-diff {
  flex: 1;
  min-height: 0;
}
</style>
