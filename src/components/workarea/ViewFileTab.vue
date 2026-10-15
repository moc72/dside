<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import Icon from "../Icon.vue";
import FileTree from "../FileTree.vue";
import MonacoEditor from "../MonacoEditor.vue";
import { t } from "../../i18n";
import type { FileNode } from "../../types";
import { languageFromPath } from "../../lib/language";
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
      <MonacoEditor
        v-if="selectedPath && !error"
        :path="selectedPath"
        :model-value="content"
        :language="language"
        :theme="editorTheme"
        :options="viewerOptions"
      />
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
