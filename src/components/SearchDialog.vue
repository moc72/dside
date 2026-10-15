<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Icon from "./Icon.vue";
import { t } from "../i18n";
import { closeSearch } from "../stores/ui";
import { openSession } from "../composables/useEngine";

interface SearchHit {
  kind: string;
  sessionId: string;
  sessionTitle: string;
  snippet: string;
}

const query = ref("");
const hits = ref<SearchHit[]>([]);
const input = ref<HTMLInputElement | null>(null);
let timer: number | undefined;

async function run() {
  const value = query.value.trim();
  if (!value) {
    hits.value = [];
    return;
  }
  try {
    hits.value = await invoke<SearchHit[]>("search", { query: value, limit: 40 });
  } catch {
    hits.value = [];
  }
}

watch(query, () => {
  if (timer) window.clearTimeout(timer);
  timer = window.setTimeout(() => void run(), 180);
});

async function pick(hit: SearchHit) {
  closeSearch();
  await openSession(hit.sessionId);
}

onMounted(() => {
  input.value?.focus();
});
</script>

<template>
  <Teleport to="body">
    <div class="backdrop" @click="closeSearch()"></div>
    <div class="panel" @click.stop>
      <div class="field">
        <Icon name="search" :size="15" />
        <input
          ref="input"
          v-model="query"
          :placeholder="t('search.placeholder')"
          @keydown.esc.prevent="closeSearch()"
          @keydown.enter.prevent="hits[0] && pick(hits[0])"
        />
        <span class="hint">{{ t("search.hint") }}</span>
      </div>

      <p v-if="!query.trim()" class="note">{{ t("search.title") }}</p>
      <p v-else-if="hits.length === 0" class="note">{{ t("search.empty") }}</p>

      <ul v-else class="hits">
        <li v-for="(hit, index) in hits" :key="`${hit.sessionId}-${index}`">
          <button class="hit" type="button" @click="pick(hit)">
            <span class="badge">{{ hit.kind === "session" ? t("search.session") : t("search.message") }}</span>
            <span class="body">
              <span class="title">{{ hit.sessionTitle || t("common.unknown") }}</span>
              <span class="snippet">{{ hit.snippet }}</span>
            </span>
          </button>
        </li>
      </ul>
    </div>
  </Teleport>
</template>

<style scoped>
.backdrop {
  position: fixed;
  inset: 0;
  z-index: 60;
  background: var(--bg-overlay);
}

.panel {
  position: fixed;
  top: 12vh;
  left: 50%;
  transform: translateX(-50%);
  z-index: 61;
  width: min(620px, calc(100vw - 40px));
  max-height: 66vh;
  display: flex;
  flex-direction: column;
  border: 1px solid var(--border-strong);
  border-radius: 12px;
  background: var(--bg-elevated);
  box-shadow: 0 24px 60px rgba(0, 0, 0, 0.6);
  overflow: hidden;
}

.field {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 14px;
  border-bottom: 1px solid var(--border);
  color: var(--text-muted);
}

.field input {
  flex: 1;
  min-width: 0;
  border: none;
  outline: none;
  background: transparent;
  color: var(--text-strong);
  font-size: 14px;
}

.hint {
  font-size: 11px;
  opacity: 0.7;
}

.note {
  margin: 0;
  padding: 16px 14px;
  color: var(--text-muted);
  font-size: 12px;
}

.hits {
  list-style: none;
  margin: 0;
  padding: 6px;
  overflow-y: auto;
}

.hit {
  display: flex;
  gap: 10px;
  width: 100%;
  padding: 8px 10px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text);
  text-align: left;
}

.hit:hover {
  background: var(--bg-hover);
}

.badge {
  flex: none;
  align-self: flex-start;
  margin-top: 2px;
  padding: 1px 7px;
  border-radius: 999px;
  background: var(--accent-strong);
  color: var(--accent-contrast);
  font-size: 10px;
}

.body {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.title {
  color: var(--text-strong);
  font-size: 12px;
}

.snippet {
  color: var(--text-muted);
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
