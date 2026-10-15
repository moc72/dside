<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Icon from "../Icon.vue";
import { t } from "../../i18n";

interface MemoryRow {
  id: number;
  kind: string;
  scope: string;
  text: string;
  importance: number;
  createdAt: number;
}

const rows = ref<MemoryRow[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);

const scopeLabels = computed<Record<string, string>>(() => ({
  user: t("memory.scopeUser"),
  agent: t("memory.scopeAgent"),
  session: t("memory.scopeSession"),
}));

async function load() {
  loading.value = true;
  error.value = null;
  try {
    rows.value = await invoke<MemoryRow[]>("memories_list", { limit: 200 });
  } catch (cause) {
    error.value = String(cause);
    rows.value = [];
  } finally {
    loading.value = false;
  }
}

async function forget(id: number) {
  try {
    await invoke("memory_delete", { id });
  } finally {
    await load();
  }
}

function formatDate(seconds: number): string {
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "short",
    timeStyle: "short",
  }).format(new Date(seconds * 1000));
}

onMounted(() => {
  void load();
});
</script>

<template>
  <div class="memory">
    <header class="head">
      <div>
        <h2 class="title">{{ t("memory.title") }}</h2>
        <p class="subtitle">{{ t("memory.hint") }}</p>
      </div>
      <button class="mini" type="button" :title="t('memory.refresh')" @click="load">
        <Icon name="reload" :size="14" />
      </button>
    </header>

    <p v-if="loading" class="note">{{ t("viewFile.loading") }}</p>
    <p v-else-if="error" class="note error">{{ error }}</p>
    <p v-else-if="rows.length === 0" class="note">{{ t("memory.empty") }}</p>

    <ul v-else class="list">
      <li v-for="row in rows" :key="row.id" class="entry">
        <div class="entry-head">
          <span class="chip scope">{{ scopeLabels[row.scope] ?? row.scope }}</span>
          <span class="chip kind">{{ row.kind }}</span>
          <span class="when">{{ formatDate(row.createdAt) }}</span>
          <button
            class="mini"
            type="button"
            :title="t('memory.forget')"
            @click="forget(row.id)"
          >
            <Icon name="x" :size="12" />
          </button>
        </div>
        <p class="text">{{ row.text }}</p>
      </li>
    </ul>
  </div>
</template>

<style scoped>
.memory {
  height: 100%;
  overflow-y: auto;
  padding: 16px 14px;
}

.head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 14px;
}

.title {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--text-strong);
}

.subtitle {
  margin: 4px 0 0;
  color: var(--text-muted);
  font-size: 12px;
}

.mini {
  display: grid;
  place-items: center;
  width: 24px;
  height: 24px;
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

.note {
  color: var(--text-muted);
  font-size: 12px;
}

.note.error {
  color: var(--danger);
}

.list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.entry {
  padding: 10px 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg-panel);
}

.entry-head {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 6px;
}

.chip {
  padding: 1px 7px;
  border-radius: 999px;
  font-size: 11px;
}

.chip.scope {
  background: var(--accent-strong);
  color: var(--accent-contrast);
}

.chip.kind {
  border: 1px solid var(--border-strong);
  color: var(--text-muted);
}

.when {
  margin-left: auto;
  color: var(--text-muted);
  font-size: 11px;
}

.text {
  margin: 0;
  color: var(--text);
  font-size: 12px;
  line-height: 1.5;
}
</style>
