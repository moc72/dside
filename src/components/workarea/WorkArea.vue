<script setup lang="ts">
import Icon from "../Icon.vue";
import { t } from "../../i18n";
import type { WorkAreaTabKind } from "../../types";
import { activeTab, activateTab, closeTab, tabs } from "../../stores/workArea";
import ProgressTab from "./ProgressTab.vue";
import ViewFileTab from "./ViewFileTab.vue";
import BrowserTab from "./BrowserTab.vue";
import MemoryTab from "./MemoryTab.vue";
import SkillsTab from "./SkillsTab.vue";
import SchedulesTab from "./SchedulesTab.vue";

const labels: Record<
  WorkAreaTabKind,
  | "workArea.tabProgress"
  | "workArea.tabFile"
  | "workArea.tabBrowser"
  | "workArea.tabMemory"
  | "workArea.tabSkills"
  | "workArea.tabSchedules"
> = {
  progress: "workArea.tabProgress",
  file: "workArea.tabFile",
  browser: "workArea.tabBrowser",
  memory: "workArea.tabMemory",
  skills: "workArea.tabSkills",
  schedules: "workArea.tabSchedules",
};
</script>

<template>
  <aside class="work">
    <div class="tabbar">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        class="tab"
        :class="{ active: tab.id === activeTab?.id }"
        type="button"
        @click="activateTab(tab.id)"
      >
        <span class="tab-label">{{ t(labels[tab.kind]) }}</span>
        <span
          v-if="tab.kind !== 'progress'"
          class="tab-close"
          :title="t('workArea.closeTab')"
          @click.stop="closeTab(tab.id)"
        >
          <Icon name="x" :size="11" />
        </span>
      </button>

      <span class="spacer"></span>
    </div>

    <div class="content">
      <ProgressTab v-if="activeTab?.kind === 'progress'" />
      <ViewFileTab v-else-if="activeTab?.kind === 'file'" :path="activeTab.filePath ?? null" />
      <BrowserTab v-else-if="activeTab?.kind === 'browser'" />
      <MemoryTab v-else-if="activeTab?.kind === 'memory'" />
      <SkillsTab v-else-if="activeTab?.kind === 'skills'" />
      <SchedulesTab v-else-if="activeTab?.kind === 'schedules'" />
      <p v-else class="empty">{{ t("workArea.empty") }}</p>
    </div>
  </aside>
</template>

<style scoped>
.work {
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.tabbar {
  display: flex;
  align-items: center;
  gap: 4px;
  height: 36px;
  padding: 0 6px;
  border-bottom: 1px solid var(--border);
}

.tab {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 26px;
  padding: 0 10px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-muted);
  font-size: 12px;
  max-width: 160px;
}

.tab.active {
  background: var(--bg-active);
  color: var(--text-strong);
}

.tab-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tab-close {
  display: grid;
  place-items: center;
  width: 16px;
  height: 16px;
  border-radius: 3px;
}

.tab-close:hover {
  background: var(--bg-hover);
  color: var(--text-strong);
}

.spacer {
  flex: 1;
}

.content {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.empty {
  padding: 16px;
  color: var(--text-muted);
}
</style>
