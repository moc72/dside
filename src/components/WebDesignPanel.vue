<script setup lang="ts">
import { nextTick, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Icon from "./Icon.vue";
import { t } from "../i18n";
import { webOpen } from "../stores/ui";

interface BrowserWindowInfo {
  id: string;
  url: string;
}

const url = ref("");
const urlInput = ref<HTMLInputElement | null>(null);
const windows = ref<BrowserWindowInfo[]>([]);
const error = ref<string | null>(null);

/** The panel exists to open a page; opening it should put the cursor in the address line. */
watch(webOpen, (isOpen) => {
  if (isOpen) void nextTick(() => urlInput.value?.focus());
});

async function refresh() {
  try {
    windows.value = await invoke<BrowserWindowInfo[]>("browser_windows");
  } catch {
    windows.value = [];
  }
}

function normalize(value: string): string {
  const trimmed = value.trim();
  if (!trimmed) return "";
  return /^https?:\/\//i.test(trimmed) ? trimmed : `https://${trimmed}`;
}

async function open(newWindow: boolean) {
  const target = normalize(url.value);
  if (!target) return;
  url.value = target;
  error.value = null;
  try {
    await invoke("open_browser_window", { url: target, newWindow });
  } catch (cause) {
    error.value = String(cause);
  }
  await refresh();
}

async function act(id: string, action: string) {
  error.value = null;
  try {
    await invoke("browser_action", { id, action });
  } catch (cause) {
    error.value = String(cause);
  }
  await refresh();
}

async function close(id: string) {
  error.value = null;
  try {
    await invoke("browser_close", { id });
  } catch (cause) {
    error.value = String(cause);
  }
  await refresh();
}

onMounted(() => {
  void refresh();
});
</script>

<template>
  <div v-if="webOpen" class="backdrop" @click="webOpen = false"></div>

  <div v-if="webOpen" class="web">
    <div class="line">
      <input
        ref="urlInput"
        v-model="url"
        class="url"
        :placeholder="t('browser.placeholder')"
        @keydown.enter.prevent="open(false)"
      />
      <button class="mini" type="button" :title="t('browser.open')" @click="open(false)">
        <Icon name="external-link" :size="14" />
      </button>
      <button class="mini" type="button" :title="t('browser.newWindow')" @click="open(true)">
        <Icon name="plus" :size="14" />
      </button>
    </div>

    <ul v-if="windows.length" class="list">
      <li v-for="entry in windows" :key="entry.id">
        <span class="slot">{{ entry.id.replace("browser-", "") }}</span>
        <span class="target" :title="entry.url">{{ entry.url }}</span>
        <button class="mini" type="button" :title="t('browser.menu')" @click="act(entry.id, 'focus')">
          <Icon name="expand" :size="13" />
        </button>
        <button class="mini" type="button" :title="t('browser.reload')" @click="act(entry.id, 'reload')">
          <Icon name="reload" :size="13" />
        </button>
        <button class="mini" type="button" :title="t('workArea.closeTab')" @click="close(entry.id)">
          <Icon name="x" :size="13" />
        </button>
      </li>
    </ul>

    <p v-else class="hint">{{ t("browser.opensInWindow") }}</p>
    <p v-if="error" class="error">{{ error }}</p>
  </div>
</template>

<style scoped>
.backdrop {
  position: fixed;
  inset: 0;
  z-index: 70;
}

.web {
  position: fixed;
  top: 52px;
  right: 14px;
  z-index: 71;
  width: min(440px, calc(100vw - 28px));
  padding: 10px;
  border: 1px solid var(--border-strong);
  border-radius: 12px;
  background: var(--bg-elevated);
  box-shadow: 0 18px 44px rgba(0, 0, 0, 0.55);
}

.line {
  display: flex;
  align-items: center;
  gap: 6px;
}

.url {
  flex: 1;
  min-width: 0;
  height: 28px;
  padding: 0 10px;
  border: 1px solid var(--border);
  border-radius: 14px;
  background: var(--bg-input);
  color: var(--text);
  font-size: 12px;
}

.url:focus {
  outline: none;
  border-color: var(--accent);
}

.mini {
  display: grid;
  place-items: center;
  width: 26px;
  height: 26px;
  flex: none;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-muted);
}

.mini:hover {
  background: var(--bg-hover);
  color: var(--text-strong);
}

.list {
  list-style: none;
  margin: 10px 0 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.list li {
  display: flex;
  align-items: center;
  gap: 6px;
}

.slot {
  flex: none;
  width: 14px;
  color: var(--text-muted);
  font-size: 11px;
}

.target {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text);
  font-size: 11px;
}

.hint {
  margin: 10px 2px 0;
  color: var(--text-muted);
  font-size: 11px;
  line-height: 1.5;
}

.error {
  margin: 8px 2px 0;
  color: var(--danger);
  font-size: 11px;
}
</style>
