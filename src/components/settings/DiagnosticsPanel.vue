<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Icon from "../Icon.vue";
import { t } from "../../i18n";
import {
  balance,
  balanceError,
  knownModels,
  loadBalance,
  loadModels,
  modelsError,
} from "../../stores/models";

interface Diagnostics {
  appVersion: string;
  os: string;
  arch: string;
  keyringPresent: boolean;
  keyringBackend: string;
  workspace: string;
  baseUrl: string;
  model: string;
  databasePath: string;
  databaseBytes: number;
  editors: string[];
}

const info = ref<Diagnostics | null>(null);
const loading = ref(false);
const error = ref<string | null>(null);
const testing = ref(false);
const testResult = ref<string | null>(null);
const copied = ref(false);
let copiedTimer: number | undefined;

const keyringText = computed(() => {
  if (!info.value) return "—";
  if (!info.value.keyringPresent) return t("diagnostics.missing");
  return `${t("diagnostics.present")} · ${info.value.keyringBackend}`;
});

const databaseText = computed(() => {
  if (!info.value) return "—";
  const bytes = info.value.databaseBytes;
  const size =
    bytes >= 1024 * 1024
      ? `${(bytes / (1024 * 1024)).toFixed(1)} MB`
      : `${Math.max(1, Math.round(bytes / 1024))} KB`;
  return `${info.value.databasePath} · ${size}`;
});

async function refresh(): Promise<void> {
  loading.value = true;
  error.value = null;
  try {
    info.value = await invoke<Diagnostics>("diagnostics");
  } catch (cause) {
    error.value = String(cause);
  } finally {
    loading.value = false;
  }
}

/**
 * The one network step, and it is deliberate: models and credits are fetched serially, for
 * the same reason the settings page does it — the provider's edge resets parallel connections.
 */
async function testConnection(): Promise<void> {
  if (testing.value) return;
  testing.value = true;
  testResult.value = null;
  try {
    await loadModels();
    await loadBalance();
    if (modelsError.value || balanceError.value) {
      testResult.value = [modelsError.value, balanceError.value].filter(Boolean).join(" · ");
      return;
    }
    const models = knownModels.value.length;
    const credits = balance.value ? `${balance.value.total} ${balance.value.currency}` : "";
    testResult.value = `${t("diagnostics.testOk")} · ${models} ${t("diagnostics.models")}${
      credits ? ` · ${credits}` : ""
    }`;
  } finally {
    testing.value = false;
  }
}

function summary(): string {
  const value = info.value;
  if (!value) return "";
  return [
    `DS·IDE ${value.appVersion}`,
    `${value.os}/${value.arch}`,
    `${t("diagnostics.keyring")}: ${keyringText.value}`,
    `${t("diagnostics.workspace")}: ${value.workspace || "—"}`,
    `${t("diagnostics.provider")}: ${value.baseUrl || "—"} · ${value.model || "—"}`,
    `${t("diagnostics.database")}: ${databaseText.value}`,
    `${t("diagnostics.editors")}: ${value.editors.join(", ") || "—"}`,
  ].join("\n");
}

async function copy(): Promise<void> {
  try {
    await navigator.clipboard.writeText(summary());
    copied.value = true;
    if (copiedTimer) window.clearTimeout(copiedTimer);
    copiedTimer = window.setTimeout(() => {
      copied.value = false;
      copiedTimer = undefined;
    }, 2000);
  } catch (cause) {
    error.value = String(cause);
  }
}

onMounted(refresh);
</script>

<template>
  <div class="diagnostics">
    <h1 class="title">{{ t("diagnostics.title") }}</h1>
    <p class="hint">{{ t("diagnostics.hint") }}</p>

    <div class="actions">
      <button class="button" type="button" :disabled="loading" @click="refresh">
        <Icon name="reload" :size="14" />
        <span>{{ t("diagnostics.refresh") }}</span>
      </button>
      <button class="button" type="button" :disabled="testing" @click="testConnection">
        <Icon name="globe" :size="14" />
        <span>{{ testing ? t("diagnostics.testing") : t("diagnostics.test") }}</span>
      </button>
      <button class="button" type="button" :disabled="!info" @click="copy">
        <Icon name="doc" :size="14" />
        <span>{{ copied ? t("diagnostics.copied") : t("diagnostics.copy") }}</span>
      </button>
    </div>

    <p v-if="error" class="error">{{ error }}</p>
    <p v-if="testResult" class="test">{{ testResult }}</p>

    <div v-if="info" class="grid">
      <div class="row">
        <span class="label">{{ t("diagnostics.app") }}</span>
        <span class="value">DS·IDE {{ info.appVersion }}</span>
      </div>
      <div class="row">
        <span class="label">{{ t("diagnostics.system") }}</span>
        <span class="value">{{ info.os }} / {{ info.arch }}</span>
      </div>
      <div class="row">
        <span class="label">{{ t("diagnostics.keyring") }}</span>
        <span class="value" :class="info.keyringPresent ? 'good' : 'bad'">{{ keyringText }}</span>
      </div>
      <div class="row">
        <span class="label">{{ t("diagnostics.workspace") }}</span>
        <span class="value mono">{{ info.workspace || "—" }}</span>
      </div>
      <div class="row">
        <span class="label">{{ t("diagnostics.provider") }}</span>
        <span class="value mono">{{ info.baseUrl || "—" }} · {{ info.model || "—" }}</span>
      </div>
      <div class="row">
        <span class="label">{{ t("diagnostics.database") }}</span>
        <span class="value mono">{{ databaseText }}</span>
      </div>
      <div class="row">
        <span class="label">{{ t("diagnostics.editors") }}</span>
        <span class="value">{{ info.editors.join(", ") || "—" }}</span>
      </div>
    </div>
    <p v-else-if="loading" class="hint">{{ t("viewFile.loading") }}</p>

    <pre v-if="info" class="dump">{{ summary() }}</pre>
  </div>
</template>

<style scoped>
.diagnostics {
  max-width: 780px;
}

.title {
  margin: 0 0 6px;
  color: var(--text-strong);
  font-size: 20px;
}

.hint {
  margin: 0 0 18px;
  color: var(--text-muted);
  font-size: 13px;
  line-height: 1.5;
}

.actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 14px;
}

.button {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  height: 30px;
  padding: 0 12px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  background: var(--bg-panel);
  color: var(--text-strong);
  font-size: 12px;
}

.button:hover:not(:disabled) {
  background: var(--bg-hover);
}

.button:disabled {
  opacity: 0.5;
}

.error {
  color: var(--danger);
  font-size: 12px;
  white-space: pre-wrap;
}

.test {
  margin: 0 0 12px;
  padding: 8px 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-panel);
  color: var(--text);
  font-size: 12px;
}

.grid {
  display: flex;
  flex-direction: column;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg-panel);
  overflow: hidden;
}

.row {
  display: grid;
  grid-template-columns: 170px minmax(0, 1fr);
  gap: 12px;
  padding: 9px 12px;
  border-bottom: 1px solid var(--border);
  font-size: 12px;
}

.row:last-child {
  border-bottom: none;
}

.label {
  color: var(--text-muted);
}

.value {
  min-width: 0;
  color: var(--text);
  overflow-wrap: anywhere;
}

.value.mono {
  font-family: var(--mono);
  font-size: 11.5px;
}

.good {
  color: var(--accent);
}

.bad {
  color: var(--danger);
}

.dump {
  margin: 14px 0 0;
  padding: 10px 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg);
  color: var(--text-muted);
  font-family: var(--mono);
  font-size: 11px;
  line-height: 1.5;
  white-space: pre-wrap;
  user-select: text;
}
</style>
