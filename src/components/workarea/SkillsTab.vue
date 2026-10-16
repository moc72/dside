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

/** The skill being written: either an existing one opened, or a new one. */
const draftName = ref("");
const draftDescription = ref("");
const draftBody = ref("");
const editingPath = ref<string | null>(null);
const saving = ref(false);
const note = ref<string | null>(null);

function startNew() {
  editingPath.value = null;
  draftName.value = "";
  draftDescription.value = "";
  draftBody.value = "# Wann hilft dieser Skill\n\n# Schritte\n\n1. ";
  note.value = null;
}

/**
 * Opens a skill for editing. The stored file starts with front matter; the description lives
 * there and the body follows, so the two fields of the form can be filled from it.
 */
async function edit(skill: SkillSummary) {
  try {
    const raw = await invoke<string>("skill_read", { path: skill.path });
    const match = /^---\n([\s\S]*?)\n---\n?/.exec(raw);
    const front = match?.[1] ?? "";
    draftName.value = skill.name;
    draftDescription.value =
      front
        .split("\n")
        .find((line) => line.trim().startsWith("description:"))
        ?.split(":")
        .slice(1)
        .join(":")
        .trim() ?? skill.description;
    draftBody.value = match ? raw.slice(match[0].length).trim() : raw.trim();
    editingPath.value = skill.path;
    note.value = null;
  } catch (cause) {
    note.value = String(cause);
  }
}

async function save() {
  if (!draftName.value.trim()) return;
  saving.value = true;
  note.value = null;
  try {
    const path = await invoke<string>("skill_write", {
      name: draftName.value,
      description: draftDescription.value,
      content: draftBody.value,
    });
    editingPath.value = path;
    await load();
  } catch (cause) {
    note.value = String(cause);
  } finally {
    saving.value = false;
  }
}

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
        <button class="mini" type="button" :title="t('skills.new')" @click="startNew">
          <Icon name="plus" :size="14" />
        </button>
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
        <button class="entry-open" type="button" @click="edit(skill)">
          <span class="entry-head">
            <Icon name="doc" :size="13" />
            <span class="name">{{ skill.name }}</span>
          </span>
          <span v-if="skill.description" class="desc">{{ skill.description }}</span>
        </button>
      </li>
    </ul>

    <!-- Writing a skill is editing a file; the panel is only a comfortable way to do it. -->
    <section class="editor">
      <div class="field">
        <label for="skill-name">{{ t("skills.name") }}</label>
        <input id="skill-name" v-model="draftName" class="input" :placeholder="t('skills.namePlaceholder')" />
      </div>
      <div class="field">
        <label for="skill-desc">{{ t("skills.description") }}</label>
        <input
          id="skill-desc"
          v-model="draftDescription"
          class="input"
          :placeholder="t('skills.descriptionPlaceholder')"
        />
      </div>
      <textarea v-model="draftBody" class="body" rows="8" :placeholder="t('skills.bodyPlaceholder')"></textarea>
      <div class="editor-actions">
        <span class="path">{{ editingPath ?? t("skills.unsaved") }}</span>
        <button class="mini-text" type="button" :disabled="saving || !draftName.trim()" @click="save">
          {{ t("skills.save") }}
        </button>
      </div>
      <p v-if="note" class="note error">{{ note }}</p>
    </section>
  </div>
</template>

<style scoped>
.entry-open {
  display: block;
  width: 100%;
  border: none;
  background: transparent;
  color: inherit;
  text-align: left;
  cursor: pointer;
}

.entry-open:hover .name {
  color: var(--text-strong);
}

.editor {
  margin-top: 16px;
  padding-top: 14px;
  border-top: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.field label {
  color: var(--text-muted);
  font-size: 11px;
}

.input,
.body {
  width: 100%;
  padding: 7px 9px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  background: var(--bg);
  color: var(--text);
  font-family: inherit;
  font-size: 12px;
}

.body {
  font-family: var(--mono);
  font-size: 11.5px;
  line-height: 1.5;
  resize: vertical;
}

.editor-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}

.mini-text {
  padding: 5px 12px;
  border: 1px solid var(--accent-strong);
  border-radius: var(--radius-sm);
  background: var(--accent-strong);
  color: var(--accent-contrast);
  font-size: 11px;
  cursor: pointer;
}

.mini-text:disabled {
  opacity: 0.5;
}

.note.error {
  color: var(--danger);
}

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
