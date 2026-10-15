<script setup lang="ts">
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Icon from "../Icon.vue";
import { t } from "../../i18n";
import { activeSessionId } from "../../stores/sessions";

interface ScheduleRow {
  id: number;
  sessionId: string;
  prompt: string;
  intervalSeconds: number;
  enabled: boolean;
  lastRunAt: number | null;
  nextRunAt: number;
}

const rows = ref<ScheduleRow[]>([]);
const draft = ref("");
const minutes = ref(60);
const loading = ref(false);

async function load() {
  loading.value = true;
  try {
    rows.value = await invoke<ScheduleRow[]>("schedules_list");
  } catch {
    rows.value = [];
  } finally {
    loading.value = false;
  }
}

async function add() {
  const prompt = draft.value.trim();
  if (!prompt) return;
  await invoke("schedule_create", {
    sessionId: activeSessionId.value,
    prompt,
    intervalSeconds: Math.max(1, Math.round(minutes.value)) * 60,
  });
  draft.value = "";
  await load();
}

async function toggle(row: ScheduleRow) {
  await invoke("schedule_toggle", { id: row.id, enabled: !row.enabled });
  await load();
}

async function remove(id: number) {
  await invoke("schedule_delete", { id });
  await load();
}

function formatTime(seconds: number | null): string {
  if (!seconds) return t("schedules.never");
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
  <div class="schedules">
    <header class="head">
      <div>
        <h2 class="title">{{ t("schedules.title") }}</h2>
        <p class="subtitle">{{ t("schedules.hint") }}</p>
      </div>
      <button class="mini" type="button" :title="t('skills.refresh')" @click="load">
        <Icon name="reload" :size="14" />
      </button>
    </header>

    <form class="form" @submit.prevent="add">
      <textarea v-model="draft" rows="2" :placeholder="t('schedules.promptPlaceholder')"></textarea>
      <div class="form-row">
        <label class="interval">
          <span>{{ t("schedules.interval") }}</span>
          <input v-model.number="minutes" type="number" min="1" max="10080" />
        </label>
        <button class="primary" type="submit" :disabled="!draft.trim()">
          <Icon name="plus" :size="14" />
          <span>{{ t("schedules.add") }}</span>
        </button>
      </div>
    </form>

    <p v-if="loading" class="note">{{ t("viewFile.loading") }}</p>
    <p v-else-if="rows.length === 0" class="note">{{ t("schedules.empty") }}</p>

    <ul v-else class="list">
      <li v-for="row in rows" :key="row.id" class="entry" :class="{ off: !row.enabled }">
        <p class="prompt">{{ row.prompt }}</p>
        <div class="meta">
          <span>⏱ {{ Math.round(row.intervalSeconds / 60) }} min</span>
          <span>{{ t("schedules.next") }}: {{ formatTime(row.nextRunAt) }}</span>
          <span>{{ t("schedules.last") }}: {{ formatTime(row.lastRunAt) }}</span>
        </div>
        <div class="entry-actions">
          <button class="mini" type="button" :title="t('schedules.enable')" @click="toggle(row)">
            <Icon :name="row.enabled ? 'check' : 'x'" :size="13" />
          </button>
          <button class="mini" type="button" :title="t('schedules.remove')" @click="remove(row.id)">
            <Icon name="x" :size="13" />
          </button>
        </div>
      </li>
    </ul>
  </div>
</template>

<style scoped>
.schedules {
  height: 100%;
  overflow-y: auto;
  padding: 16px 14px;
}

.head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 12px;
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

.form {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg-panel);
  margin-bottom: 14px;
}

.form textarea {
  resize: none;
  border: none;
  outline: none;
  background: transparent;
  color: var(--text);
  font: inherit;
  font-size: 12px;
}

.form-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}

.interval {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--text-muted);
  font-size: 12px;
}

.interval input {
  width: 72px;
  height: 26px;
  padding: 0 6px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-input);
  color: var(--text);
}

.primary {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 26px;
  padding: 0 10px;
  border: none;
  border-radius: var(--radius-sm);
  background: var(--accent-strong);
  color: var(--accent-contrast);
  font-size: 12px;
}

.primary:disabled {
  opacity: 0.45;
}

.note {
  color: var(--text-muted);
  font-size: 12px;
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
  position: relative;
  padding: 10px 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg-panel);
}

.entry.off {
  opacity: 0.55;
}

.prompt {
  margin: 0 0 6px;
  color: var(--text);
  font-size: 12px;
  line-height: 1.45;
  padding-right: 52px;
}

.meta {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  color: var(--text-muted);
  font-size: 11px;
}

.entry-actions {
  position: absolute;
  top: 8px;
  right: 8px;
  display: flex;
  gap: 2px;
}
</style>
