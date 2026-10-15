<script setup lang="ts">
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import Icon from "../Icon.vue";
import { t } from "../../i18n";

interface SkillSummary {
  name: string;
  description: string;
  path: string;
}

const skills = ref<SkillSummary[]>([]);
const root = ref("");
const loading = ref(false);

async function load() {
  loading.value = true;
  try {
    skills.value = await invoke<SkillSummary[]>("skills_list");
    root.value = await invoke<string>("skills_root_path");
  } catch {
    skills.value = [];
  } finally {
    loading.value = false;
  }
}

async function openFolder() {
  if (!root.value) return;
  try {
    await revealItemInDir(root.value);
  } catch {
    // no file manager available — the path is shown in the UI anyway
  }
}

onMounted(() => {
  void load();
});
</script>

<template>
  <div class="skills">
    <header class="head">
      <div class="text">
        <h2 class="title">{{ t("skills.title") }}</h2>
        <p class="subtitle">{{ t("skills.hint", { path: root || "~/.ds-ide/skills" }) }}</p>
      </div>
      <div class="actions">
        <button class="mini" type="button" :title="t('skills.openFolder')" @click="openFolder">
          <Icon name="folder" :size="14" />
        </button>
        <button class="mini" type="button" :title="t('skills.refresh')" @click="load">
          <Icon name="reload" :size="14" />
        </button>
      </div>
    </header>

    <p v-if="loading" class="note">{{ t("viewFile.loading") }}</p>
    <p v-else-if="skills.length === 0" class="note">{{ t("skills.empty") }}</p>

    <ul v-else class="list">
      <li v-for="skill in skills" :key="skill.path" class="entry">
        <div class="entry-head">
          <Icon name="doc" :size="13" />
          <span class="name">{{ skill.name }}</span>
        </div>
        <p v-if="skill.description" class="desc">{{ skill.description }}</p>
        <p class="path">{{ skill.path }}</p>
      </li>
    </ul>
  </div>
</template>

<style scoped>
.skills {
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
  font-size: 11px;
  word-break: break-all;
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
  gap: 8px;
}

.name {
  color: var(--text-strong);
  font-size: 12px;
  font-weight: 600;
}

.desc {
  margin: 5px 0 0;
  color: var(--text);
  font-size: 12px;
  line-height: 1.45;
}

.path {
  margin: 6px 0 0;
  color: var(--text-muted);
  font-size: 11px;
  word-break: break-all;
}
</style>
