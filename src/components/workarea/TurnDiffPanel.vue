<script setup lang="ts">
import { computed, defineAsyncComponent, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Icon from "../Icon.vue";
import { t } from "../../i18n";
import { languageFromPath } from "../../lib/language";
import { activeSessionId } from "../../stores/sessions";
import { resolvedTheme } from "../../stores/settings";
import { turnDiffRevision } from "../../stores/workArea";
import type { TurnDiff, TurnFileDiff } from "../../types";

const MonacoDiff = defineAsyncComponent(() => import("../MonacoDiff.vue"));

const diff = ref<TurnDiff | null>(null);
const selected = ref<string | null>(null);
const loading = ref(false);
const error = ref<string | null>(null);
const reverting = ref(false);
const reverted = ref(false);

const current = computed<TurnFileDiff | null>(
  () => diff.value?.files.find((file) => file.path === selected.value) ?? null,
);

const editorTheme = computed(() => (resolvedTheme.value === "light" ? "vs" : "vs-dark"));
const language = computed(() =>
  current.value ? languageFromPath(current.value.path) : "plaintext",
);

/** A diff that is already back at its pre-turn state is not worth a diff view. */
const hasChanges = computed(() => {
  if (!diff.value) return false;
  return diff.value.files.some(
    (file) => file.before !== file.after || file.existedBefore !== file.existsAfter,
  );
});

async function load(): Promise<void> {
  loading.value = true;
  error.value = null;
  try {
    const next = await invoke<TurnDiff | null>("turn_diff", {
      sessionId: activeSessionId.value,
    });
    diff.value = next;
    if (!next || !next.files.some((file) => file.path === selected.value)) {
      selected.value = next?.files[0]?.path ?? null;
    }
  } catch (cause) {
    error.value = String(cause);
    diff.value = null;
    selected.value = null;
  } finally {
    loading.value = false;
  }
}

async function undo(): Promise<void> {
  if (!diff.value?.canUndo || reverting.value) return;
  const turnId = diff.value.turnId;
  reverting.value = true;
  error.value = null;
  try {
    await invoke("turn_revert", { sessionId: activeSessionId.value, turnId });
    reverted.value = true;
    await load();
  } catch (cause) {
    error.value = `${t("turnDiff.revertFailed")} ${String(cause)}`;
  } finally {
    reverting.value = false;
  }
}

function status(file: TurnFileDiff): string {
  if (!file.existedBefore) return t("turnDiff.newFile");
  if (!file.existsAfter) return t("turnDiff.deletedFile");
  return file.origin === "write" ? t("viewFile.historyWrite") : t("viewFile.historyEdit");
}

watch([turnDiffRevision, activeSessionId], () => void load(), { immediate: true });
</script>

<template>
  <section class="changes">
    <header class="head">
      <div class="heading">
        <h3>{{ t("turnDiff.title") }}</h3>
        <p>{{ t("turnDiff.hint") }}</p>
      </div>
      <div class="actions">
        <button class="mini" type="button" :title="t('turnDiff.refresh')" @click="load">
          <Icon name="reload" :size="13" />
        </button>
        <button
          class="undo"
          type="button"
          :title="t('turnDiff.undoHint')"
          :disabled="!diff?.canUndo || reverting"
          @click="undo"
        >
          <Icon name="reload" :size="13" />
          {{ t("turnDiff.undo") }}
        </button>
      </div>
    </header>

    <p v-if="loading" class="note">{{ t("viewFile.loading") }}</p>
    <p v-else-if="error" class="note error">{{ error }}</p>
    <p v-else-if="!diff || !hasChanges" class="note">
      {{ reverted ? t("turnDiff.alreadyReverted") : t("turnDiff.empty") }}
    </p>

    <template v-else>
      <div class="body">
        <ul class="files">
          <li v-for="file in diff.files" :key="file.path">
            <button
              class="file"
              :class="{ active: file.path === selected }"
              type="button"
              @click="selected = file.path"
            >
              <span class="path">{{ file.path }}</span>
              <span class="chip">{{ status(file) }}</span>
            </button>
          </li>
        </ul>
        <div class="diff">
          <MonacoDiff
            v-if="current"
            :original="current.before"
            :modified="current.after"
            :language="language"
            :theme="editorTheme"
          />
        </div>
      </div>
      <p v-if="!diff.canUndo" class="warn">{{ t("turnDiff.undoBlocked") }}</p>
    </template>
  </section>
</template>

<style scoped>
.changes {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

.head {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 10px 12px;
  border-bottom: 1px solid var(--border);
}

.heading {
  flex: 1;
  min-width: 0;
}

.heading h3 {
  margin: 0 0 3px;
  color: var(--text-strong);
  font-size: 13px;
}

.heading p {
  margin: 0;
  color: var(--text-muted);
  font-size: 11px;
  line-height: 1.4;
}

.actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.mini {
  display: grid;
  place-items: center;
  width: 26px;
  height: 26px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-input);
  color: var(--text-muted);
  cursor: pointer;
}

.mini:hover {
  color: var(--text-strong);
  border-color: var(--accent);
}

.undo {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 26px;
  padding: 0 10px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text);
  font-size: 12px;
  cursor: pointer;
}

.undo:hover:not(:disabled) {
  border-color: var(--accent);
  color: var(--text-strong);
}

.undo:disabled {
  opacity: 0.45;
  cursor: default;
}

.body {
  display: flex;
  flex: 1;
  min-height: 0;
}

.files {
  width: 220px;
  min-width: 0;
  margin: 0;
  padding: 6px;
  border-right: 1px solid var(--border);
  list-style: none;
  overflow-y: auto;
}

.file {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 4px;
  width: 100%;
  padding: 7px 8px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: inherit;
  text-align: left;
  cursor: pointer;
}

.file:hover {
  background: var(--bg-hover);
}

.file.active {
  background: var(--bg-active);
}

.path {
  max-width: 100%;
  overflow: hidden;
  color: var(--text-strong);
  font-size: 11.5px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.chip {
  padding: 1px 6px;
  border: 1px solid var(--border);
  border-radius: 999px;
  color: var(--text-muted);
  font-size: 10px;
}

.diff {
  flex: 1;
  min-width: 0;
  min-height: 0;
}

.note {
  margin: 0;
  padding: 16px 12px;
  color: var(--text-muted);
  font-size: 12px;
}

.note.error {
  color: var(--danger);
}

.warn {
  margin: 0;
  padding: 8px 12px;
  border-top: 1px solid var(--border);
  color: var(--danger);
  font-size: 11px;
}
</style>
