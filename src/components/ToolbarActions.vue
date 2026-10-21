<script setup lang="ts">
import { onBeforeUnmount, ref } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import Icon from "./Icon.vue";
import { t } from "../i18n";
import { settings } from "../stores/settings";
import { openTab, toggleWorkArea } from "../stores/workArea";
import { toggleWeb } from "../stores/ui";
import {
  activeEditor,
  detectEditors,
  detecting,
  editors,
  openInEditor,
  useEditors,
} from "../composables/useEditors";
import { everydayMode } from "../engine/mode";

const EDITOR_DOWNLOAD_URL = "https://code.visualstudio.com/download";

// Looks for installed editors once, when the toolbar appears. Without this call the
// dropdown stays on "no editor found" forever — a missing side effect, not a missing scan.
useEditors();

const menuOpen = ref(false);
const openError = ref<string | null>(null);
let errorTimer: number | undefined;

function clearError(): void {
  if (errorTimer !== undefined) {
    window.clearTimeout(errorTimer);
    errorTimer = undefined;
  }
  openError.value = null;
}

function showError(message: string): void {
  clearError();
  openError.value = message;
  errorTimer = window.setTimeout(() => {
    errorTimer = undefined;
    openError.value = null;
  }, 5000);
}

onBeforeUnmount(clearError);

function toggleMenu(): void {
  clearError();
  menuOpen.value = !menuOpen.value;
}

function choose(id: string) {
  settings.pinnedEditorId = id;
  menuOpen.value = false;
  clearError();
}

async function openWorkspace(): Promise<void> {
  // Without an editor the picker is the only useful action — detection and download live there.
  if (!activeEditor.value) {
    menuOpen.value = true;
    return;
  }
  menuOpen.value = false;
  clearError();
  try {
    await openInEditor();
  } catch (error) {
    showError(String(error));
  }
}

function detect() {
  clearError();
  void detectEditors();
}

async function openDownload() {
  menuOpen.value = false;
  clearError();
  await openUrl(EDITOR_DOWNLOAD_URL);
}
</script>

<template>
  <div class="toolbar">
    <!-- Everyday mode has no code editor to hand things to, so the picker goes away. -->
    <div v-if="!everydayMode" class="editor-wrap">
      <div class="editor">
        <button
          class="editor-open"
          type="button"
          :title="activeEditor ? t('toolbar.editorOpen') : t('toolbar.editorChoose')"
          @click="openWorkspace"
        >
          <span class="logo">&lt;/&gt;</span>
          <span class="label">
            {{
              activeEditor?.label ??
              (detecting ? t("toolbar.editorSearching") : t("toolbar.editorNone"))
            }}
          </span>
          <Icon v-if="activeEditor" name="external-link" :size="12" class="open-hint" />
        </button>
        <button
          class="editor-toggle"
          type="button"
          :title="t('toolbar.editorChoose')"
          :aria-label="t('toolbar.editorChoose')"
          :aria-expanded="menuOpen"
          @click="toggleMenu"
        >
          <Icon :name="menuOpen ? 'chevron-up' : 'chevron-down'" :size="14" />
        </button>
      </div>

      <div v-if="menuOpen" class="backdrop" @click="menuOpen = false"></div>
      <div v-if="menuOpen" class="menu">
        <button
          v-for="editor in editors"
          :key="editor.exec"
          class="menu-item"
          type="button"
          :title="
            editor.id === settings.pinnedEditorId
              ? t('toolbar.editorPinActive')
              : t('toolbar.editorPin')
          "
          @click="choose(editor.id)"
        >
          <Icon
            :name="editor.id === settings.pinnedEditorId ? 'check' : 'doc'"
            :size="14"
          />
          <span class="grow">{{ editor.label }}</span>
          <span class="source">{{ editor.source }}</span>
        </button>

        <p v-if="editors.length === 0" class="menu-empty">{{ t("toolbar.editorNone") }}</p>

        <div class="separator"></div>

        <button class="menu-item" type="button" @click="detect">
          <Icon name="reload" :size="14" />
          <span class="grow">{{ t("toolbar.editorDetect") }}</span>
        </button>
        <button class="menu-item" type="button" @click="openDownload">
          <Icon name="external-link" :size="14" />
          <span class="grow">{{ t("toolbar.editorDownload") }}</span>
        </button>
      </div>

      <p v-if="openError" class="editor-error">
        {{ t("toolbar.editorOpenFailed") }}: {{ openError }}
      </p>
    </div>

    <button class="icon-btn" type="button" :title="t('toolbar.browser')" @click="toggleWeb">
      <Icon name="globe" />
    </button>
    <button class="icon-btn" type="button" :title="t('toolbar.viewFile')" @click="openTab('file')">
      <Icon name="folder" />
    </button>
    <button
      class="icon-btn"
      type="button"
      :title="t('toolbar.toggleWorkArea')"
      @click="toggleWorkArea"
    >
      <Icon name="panel" />
    </button>
  </div>
</template>

<style scoped>
.toolbar {
  display: flex;
  align-items: center;
  gap: 4px;
}

.editor-wrap {
  position: relative;
}

.editor {
  display: flex;
  align-items: stretch;
  height: 28px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-panel);
  overflow: hidden;
}

.editor-open,
.editor-toggle {
  display: flex;
  align-items: center;
  border: none;
  background: transparent;
  color: var(--text-strong);
  font: inherit;
}

.editor-open {
  gap: 8px;
  padding: 0 6px 0 10px;
  font-size: 12px;
}

.editor-open:hover,
.editor-toggle:hover {
  background: var(--bg-hover);
}

.editor-toggle {
  display: grid;
  place-items: center;
  width: 26px;
  border-left: 1px solid var(--border);
  color: var(--text-muted);
}

/* Narrow window: the logo and the chevron still open the menu, the label is the only part
   that has to go. Without this the right-hand toolbar pushed the session title out. */
@media (max-width: 980px) {
  .editor-open .label {
    display: none;
  }

  .editor-open {
    padding: 0 8px;
  }
}

.logo {
  color: var(--accent);
  font-weight: 700;
  font-size: 11px;
}

.label {
  max-width: 190px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.open-hint {
  color: var(--text-muted);
}

.icon-btn {
  display: grid;
  place-items: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-muted);
}

.icon-btn:hover {
  background: var(--bg-hover);
  color: var(--text-strong);
}

.backdrop {
  position: fixed;
  inset: 0;
  z-index: 30;
}

.menu {
  position: absolute;
  top: 34px;
  right: 0;
  z-index: 31;
  width: 300px;
  padding: 6px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg-panel);
  box-shadow: 0 14px 36px rgba(0, 0, 0, 0.45);
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 7px 10px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text);
  text-align: left;
}

.menu-item:hover {
  background: var(--bg-hover);
}

.grow {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.source {
  color: var(--text-muted);
  font-size: 11px;
}

.menu-empty {
  margin: 6px 10px;
  color: var(--text-muted);
}

.separator {
  height: 1px;
  margin: 6px 4px;
  background: var(--border);
}

.editor-error {
  position: absolute;
  top: 34px;
  right: 0;
  z-index: 32;
  max-width: 340px;
  padding: 7px 9px;
  border: 1px solid var(--danger);
  border-radius: var(--radius-sm);
  background: var(--bg-elevated);
  color: var(--danger);
  font-size: 11px;
  line-height: 1.35;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.4);
}
</style>
