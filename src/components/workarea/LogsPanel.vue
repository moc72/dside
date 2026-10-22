<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { t } from "../../i18n";

/**
 * The companion's ears and the developer's window into the app: the same capped ring the
 * `logs` tool reads. Polling is enough here — the tab is local and the log is small.
 */
const lines = ref<string[]>([]);
const error = ref<string | null>(null);
const text = ref<HTMLElement | null>(null);

const PAGE = 250;
let timer: number | undefined;
let busy = false;

async function refresh(): Promise<void> {
  if (busy) return;
  busy = true;
  try {
    const stick = text.value
      ? text.value.scrollTop + text.value.clientHeight >= text.value.scrollHeight - 24
      : true;
    lines.value = await invoke<string[]>("log_tail", { limit: PAGE });
    error.value = null;
    await nextTick();
    if (stick && text.value) text.value.scrollTop = text.value.scrollHeight;
  } catch (cause) {
    error.value = String(cause);
  } finally {
    busy = false;
  }
}

onMounted(() => {
  void refresh();
  timer = window.setInterval(refresh, 1500);
});

onBeforeUnmount(() => {
  if (timer) window.clearInterval(timer);
});
</script>

<template>
  <div class="logs-panel">
    <div class="logs-bar">
      <span class="logs-title">{{ t("logs.title") }}</span>
      <span class="logs-count">{{ lines.length }}</span>
      <span class="spacer"></span>
      <button class="mini-text" type="button" @click="refresh">
        {{ t("logs.refresh") }}
      </button>
    </div>
    <p v-if="error" class="error">{{ error }}</p>
    <pre v-else-if="lines.length" ref="text" class="logs-text">{{ lines.join("\n") }}</pre>
    <p v-else class="empty">{{ t("logs.empty") }}</p>
  </div>
</template>

<style scoped>
.logs-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

.logs-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 32px;
  padding: 0 8px;
  border-bottom: 1px solid var(--border);
  color: var(--text-muted);
  font-size: 11px;
}

.logs-title {
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.logs-count {
  color: var(--text-muted);
}

.spacer {
  flex: 1;
}

.logs-text {
  flex: 1;
  min-height: 0;
  margin: 0;
  padding: 6px 8px;
  overflow: auto;
  color: var(--text);
  font-family: var(--mono);
  font-size: 11px;
  line-height: 1.45;
  white-space: pre-wrap;
  word-break: break-word;
}

.error {
  margin: 8px;
  color: var(--danger);
  font-size: 12px;
}

.empty {
  margin: 16px;
  color: var(--text-muted);
  font-size: 12px;
}
</style>
