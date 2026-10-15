<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Icon from "../Icon.vue";
import { t } from "../../i18n";

const url = ref("");
const error = ref<string | null>(null);

async function call(command: string, args?: Record<string, unknown>) {
  error.value = null;
  try {
    await invoke(command, args);
  } catch (cause) {
    error.value = String(cause);
  }
}

function normalize(value: string): string {
  const trimmed = value.trim();
  if (!trimmed) return "";
  return /^https?:\/\//i.test(trimmed) ? trimmed : `https://${trimmed}`;
}

async function open() {
  const target = normalize(url.value);
  if (!target) return;
  url.value = target;
  await call("open_browser_window", { url: target });
}
</script>

<template>
  <div class="browser">
    <div class="bar">
      <button
        class="icon"
        type="button"
        :title="t('browser.back')"
        @click="call('browser_action', { action: 'back' })"
      >
        <Icon name="arrow-left" :size="14" />
      </button>
      <button
        class="icon"
        type="button"
        :title="t('browser.forward')"
        @click="call('browser_action', { action: 'forward' })"
      >
        <Icon name="arrow-right" :size="14" />
      </button>
      <button
        class="icon"
        type="button"
        :title="t('browser.reload')"
        @click="call('browser_action', { action: 'reload' })"
      >
        <Icon name="reload" :size="14" />
      </button>

      <input
        v-model="url"
        class="url"
        :placeholder="t('browser.placeholder')"
        @keydown.enter.prevent="open"
      />

      <button
        class="icon"
        type="button"
        :title="t('browser.menu')"
        @click="call('browser_action', { action: 'focus' })"
      >
        <Icon name="external-link" :size="14" />
      </button>
    </div>

    <div class="body">
      <Icon name="globe" :size="30" />
      <p class="hint">{{ t("browser.opensInWindow") }}</p>
      <button class="primary" type="button" :disabled="!url.trim()" @click="open">
        {{ t("browser.open") }}
      </button>
      <p v-if="error" class="error">{{ error }}</p>
    </div>
  </div>
</template>

<style scoped>
.browser {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.bar {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 8px 10px;
  border-bottom: 1px solid var(--border);
}

.icon {
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

.icon:hover {
  background: var(--bg-hover);
  color: var(--text-strong);
}

.url {
  flex: 1;
  min-width: 0;
  height: 28px;
  padding: 0 12px;
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

.body {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 20px;
  color: var(--text-muted);
  text-align: center;
}

.hint {
  margin: 0;
  max-width: 320px;
  font-size: 12px;
  line-height: 1.5;
}

.primary {
  height: 28px;
  padding: 0 14px;
  border: none;
  border-radius: var(--radius-sm);
  background: var(--accent-strong);
  color: var(--accent-contrast);
  font-size: 12px;
}

.primary:disabled {
  opacity: 0.45;
}

.error {
  margin: 0;
  max-width: 340px;
  color: var(--danger);
  font-size: 12px;
}
</style>
