<script setup lang="ts">
import { ref } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import Icon from "./Icon.vue";
import { t } from "../i18n";
import { settings } from "../stores/settings";
import { openTab, toggleWorkArea } from "../stores/workArea";
import { activeEditor, detectEditors, detecting, editors } from "../composables/useEditors";

const EDITOR_DOWNLOAD_URL = "https://code.visualstudio.com/download";

const menuOpen = ref(false);

function choose(id: string) {
  settings.pinnedEditorId = id;
  menuOpen.value = false;
}

function detect() {
  void detectEditors();
}

async function openDownload() {
  menuOpen.value = false;
  await openUrl(EDITOR_DOWNLOAD_URL);
}
</script>

<template>
  <div class="toolbar">
    <div class="editor-wrap">
      <button class="editor" type="button" @click="menuOpen = !menuOpen">
        <span class="logo">&lt;/&gt;</span>
        <span class="label">
          {{
            activeEditor?.label ??
            (detecting ? t("toolbar.editorSearching") : t("toolbar.editorNone"))
          }}
        </span>
        <Icon name="chevron-down" :size="14" />
      </button>

      <div v-if="menuOpen" class="backdrop" @click="menuOpen = false"></div>
      <div v-if="menuOpen" class="menu">
        <button
          v-for="editor in editors"
          :key="editor.exec"
          class="menu-item"
          type="button"
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
    </div>

    <button class="icon-btn" type="button" :title="t('toolbar.browser')" @click="openTab('browser')">
      <Icon name="globe" />
    </button>
    <button class="icon-btn" type="button" :title="t('toolbar.viewFile')" @click="openTab('file')">
      <Icon name="folder" />
    </button>
    <button
      class="icon-btn"
      type="button"
      :title="t('toolbar.memory')"
      @click="openTab('memory')"
    >
      <Icon name="sparkle" />
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
  align-items: center;
  gap: 8px;
  height: 28px;
  padding: 0 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-panel);
  color: var(--text-strong);
  font-size: 12px;
}

.editor:hover {
  background: var(--bg-hover);
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
</style>
