<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Icon from "../Icon.vue";
import { t } from "../../i18n";

/**
 * Seeky's long-term memory, seen from the user's side.
 *
 * It lives in the settings rather than on the work area because it is not a tool: it is the
 * place where the user corrects what their partner believes about them. Nothing here deletes
 * a row — an entry the user takes out is retired and can be put back.
 */
interface MemoryRow {
  id: number;
  kind: string;
  scope: string;
  text: string;
  importance: number;
  createdAt: number;
  origin: string;
  editedAt: number | null;
  retiredAt: number | null;
}

interface MemoryReport {
  total: number;
  duplicates: number[];
  stale: number[];
}

const rows = ref<MemoryRow[]>([]);
const retired = ref<MemoryRow[]>([]);
const report = ref<MemoryReport | null>(null);
const loading = ref(false);
const error = ref<string | null>(null);

const editingId = ref<number | null>(null);
const draft = ref("");
const retiredOpen = ref(false);

const scopeLabels = computed<Record<string, string>>(() => ({
  user: t("memory.scopeUser"),
  agent: t("memory.scopeAgent"),
  session: t("memory.scopeSession"),
}));

async function load() {
  loading.value = true;
  error.value = null;
  try {
    const [active, aside] = await Promise.all([
      invoke<MemoryRow[]>("memories_list", { limit: 200 }),
      invoke<MemoryRow[]>("memories_retired", { limit: 200 }),
    ]);
    rows.value = active;
    retired.value = aside;
  } catch (cause) {
    error.value = String(cause);
    rows.value = [];
    retired.value = [];
  } finally {
    loading.value = false;
  }
}

function startEdit(row: MemoryRow) {
  editingId.value = row.id;
  draft.value = row.text;
}

function cancelEdit() {
  editingId.value = null;
  draft.value = "";
}

/** Saving a correction is the one thing that marks an entry as the user's own. */
async function saveEdit(id: number) {
  const text = draft.value.trim();
  if (!text) return;
  try {
    await invoke("memory_update", { id, text });
    cancelEdit();
  } catch (cause) {
    error.value = String(cause);
  }
  await load();
}

async function retire(id: number) {
  try {
    await invoke("memory_retire", { id });
  } catch (cause) {
    error.value = String(cause);
  }
  await load();
}

async function restore(id: number) {
  try {
    await invoke("memory_restore", { id });
  } catch (cause) {
    error.value = String(cause);
  }
  await load();
}

function formatDate(seconds: number): string {
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "short",
    timeStyle: "short",
  }).format(new Date(seconds * 1000));
}

/** The "doctor" pass: mechanical findings only. Setting entries aside stays a human decision. */
async function check() {
  try {
    report.value = await invoke<MemoryReport>("memory_report");
  } catch {
    report.value = null;
  }
}

async function prune(ids: number[]) {
  if (ids.length === 0) return;
  await invoke("memory_prune", { ids });
  report.value = null;
  await load();
}

onMounted(() => {
  void load();
});
</script>

<template>
  <div class="memory">
    <header class="head">
      <p class="hint">{{ t("memory.hint") }}</p>
      <div class="actions">
        <button class="mini" type="button" :title="t('memory.check')" @click="check">
          <Icon name="sparkle" :size="14" />
        </button>
        <button class="mini" type="button" :title="t('memory.refresh')" @click="load">
          <Icon name="reload" :size="14" />
        </button>
      </div>
    </header>

    <div v-if="report" class="report">
      <div v-if="report.duplicates.length || report.stale.length">
        <div v-if="report.duplicates.length" class="report-row">
          <span>{{ t("memory.duplicates", { count: report.duplicates.length }) }}</span>
          <button class="mini-text" type="button" @click="prune(report.duplicates)">
            {{ t("memory.retire") }}
          </button>
        </div>
        <div v-if="report.stale.length" class="report-row">
          <span>{{ t("memory.stale", { count: report.stale.length }) }}</span>
          <button class="mini-text" type="button" @click="prune(report.stale)">
            {{ t("memory.retire") }}
          </button>
        </div>
      </div>
      <p v-else class="report-clean">{{ t("memory.clean") }}</p>
    </div>

    <p v-if="loading" class="note">{{ t("viewFile.loading") }}</p>
    <p v-else-if="error" class="note error">{{ error }}</p>
    <p v-else-if="rows.length === 0" class="note">{{ t("memory.empty") }}</p>

    <ul v-else class="list">
      <li v-for="row in rows" :key="row.id" class="entry">
        <div class="entry-head">
          <span class="chip scope">{{ scopeLabels[row.scope] ?? row.scope }}</span>
          <span class="chip kind">{{ row.kind }}</span>
          <span v-if="row.origin === 'user'" class="chip mine">{{ t("memory.changedByUser") }}</span>
          <span class="when">{{ formatDate(row.editedAt ?? row.createdAt) }}</span>
          <button
            class="mini"
            type="button"
            :title="t('memory.edit')"
            @click="startEdit(row)"
          >
            <Icon name="pencil" :size="12" />
          </button>
          <button
            class="mini"
            type="button"
            :title="t('memory.retire')"
            @click="retire(row.id)"
          >
            <Icon name="x" :size="12" />
          </button>
        </div>

        <template v-if="editingId === row.id">
          <textarea v-model="draft" class="editor" rows="2"></textarea>
          <div class="editor-actions">
            <button class="mini-text" type="button" @click="cancelEdit">
              {{ t("memory.cancel") }}
            </button>
            <button class="mini-text primary" type="button" @click="saveEdit(row.id)">
              {{ t("memory.save") }}
            </button>
          </div>
        </template>
        <p v-else class="text">{{ row.text }}</p>
      </li>
    </ul>

    <section v-if="retired.length" class="retired">
      <button class="retired-head" type="button" @click="retiredOpen = !retiredOpen">
        <Icon :name="retiredOpen ? 'chevron-down' : 'chevron-right'" :size="12" />
        <span>{{ t("memory.retired", { count: retired.length }) }}</span>
      </button>
      <p v-if="retiredOpen" class="note">{{ t("memory.retiredHint") }}</p>
      <ul v-if="retiredOpen" class="list">
        <li v-for="row in retired" :key="row.id" class="entry aside">
          <div class="entry-head">
            <span class="chip scope">{{ scopeLabels[row.scope] ?? row.scope }}</span>
            <span class="chip kind">{{ row.kind }}</span>
            <span class="when">{{ formatDate(row.retiredAt ?? row.createdAt) }}</span>
            <button class="mini-text" type="button" @click="restore(row.id)">
              {{ t("memory.restore") }}
            </button>
          </div>
          <p class="text">{{ row.text }}</p>
        </li>
      </ul>
    </section>
  </div>
</template>

<style scoped>
.memory {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
}

.hint {
  margin: 0;
  color: var(--text-muted);
  font-size: 12px;
}

.actions {
  display: flex;
  gap: 4px;
  flex: none;
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
  margin: 0;
  color: var(--text-muted);
  font-size: 12px;
}

.note.error {
  color: var(--danger);
}

.report {
  padding: 10px 12px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius);
  background: var(--bg-panel);
}

.report-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  color: var(--text);
  font-size: 12px;
}

.report-row + .report-row {
  margin-top: 8px;
}

.report-clean {
  margin: 0;
  color: var(--text-muted);
  font-size: 12px;
}

.mini-text {
  padding: 4px 10px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text);
  font-size: 11px;
}

.mini-text:hover {
  border-color: var(--accent);
  color: var(--text-strong);
}

.mini-text.primary {
  border-color: var(--accent-strong);
  background: var(--accent-strong);
  color: var(--accent-contrast);
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

.entry.aside {
  opacity: 0.7;
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

.chip.mine {
  background: var(--bg-elevated);
  color: var(--text-strong);
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

.editor {
  width: 100%;
  padding: 6px 8px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  background: var(--bg);
  color: var(--text);
  font-family: inherit;
  font-size: 12px;
  line-height: 1.5;
  resize: vertical;
}

.editor-actions {
  display: flex;
  justify-content: flex-end;
  gap: 6px;
  margin-top: 6px;
}

.retired {
  margin-top: 6px;
}

.retired-head {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 0;
  border: none;
  background: transparent;
  color: var(--text-muted);
  font-size: 12px;
}

.retired-head:hover {
  color: var(--text-strong);
}

.retired .note {
  margin-bottom: 8px;
}
</style>
