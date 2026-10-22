<script setup lang="ts">
import { computed, defineAsyncComponent, watch } from "vue";
import Icon from "../Icon.vue";
import { t } from "../../i18n";
import type { WorkAreaTabKind } from "../../types";
import { activeTab, activateTab, attentionTabId, closeTab, openTab, tabs } from "../../stores/workArea";
import { settings } from "../../stores/settings";
import ProgressTab from "./ProgressTab.vue";
import { EVERYDAY_HIDDEN_TABS, everydayMode } from "../../engine/mode";

const ViewFileTab = defineAsyncComponent(() => import("./ViewFileTab.vue"));
const SkillsTab = defineAsyncComponent(() => import("./SkillsTab.vue"));
const SchedulesTab = defineAsyncComponent(() => import("./SchedulesTab.vue"));
const GitPanel = defineAsyncComponent(() => import("./GitPanel.vue"));
const TerminalPanel = defineAsyncComponent(() => import("../TerminalPanel.vue"));
const LogsPanel = defineAsyncComponent(() => import("./LogsPanel.vue"));
const TurnDiffPanel = defineAsyncComponent(() => import("./TurnDiffPanel.vue"));

const labels: Record<
  WorkAreaTabKind,
  | "workArea.tabProgress"
  | "workArea.tabFile"
  | "workArea.tabSkills"
  | "workArea.tabSchedules"
  | "workArea.tabTerminal"
  | "workArea.tabGit"
  | "workArea.tabLogs"
  | "workArea.tabChanges"
> = {
  progress: "workArea.tabProgress",
  file: "workArea.tabFile",
  skills: "workArea.tabSkills",
  schedules: "workArea.tabSchedules",
  terminal: "workArea.tabTerminal",
  git: "workArea.tabGit",
  logs: "workArea.tabLogs",
  changes: "workArea.tabChanges",
};

const hasTerminal = computed(() => tabs.value.some((tab) => tab.kind === "terminal"));
const hasGit = computed(() => tabs.value.some((tab) => tab.kind === "git"));
const hasLogs = computed(() => tabs.value.some((tab) => tab.kind === "logs"));
const hasChanges = computed(() => tabs.value.some((tab) => tab.kind === "changes"));

/** Everyday mode keeps the plan, skill and schedule panels out of the way. */
const shownTabs = computed(() =>
  everydayMode.value
    ? tabs.value.filter((tab) => !EVERYDAY_HIDDEN_TABS.includes(tab.kind as never))
    : tabs.value,
);

// If the open tab is one this mode hides, move to the file tab rather than showing nothing.
watch(
  [everydayMode, () => activeTab.value?.kind],
  () => {
    const kind = activeTab.value?.kind;
    if (!kind) return;
    if (everydayMode.value && EVERYDAY_HIDDEN_TABS.includes(kind as never)) {
      activateTab("file");
    }
  },
  { immediate: true },
);
</script>

<template>
  <aside class="work">
    <div class="tabbar">
      <button
        v-for="tab in shownTabs"
        :key="tab.id"
        class="tab"
        :class="{ active: tab.id === activeTab?.id, attention: tab.id === attentionTabId }"
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

      <button
        v-if="!hasTerminal"
        class="open-tool"
        type="button"
        :title="t('workArea.tabTerminal')"
        @click="openTab('terminal')"
      >
        <Icon name="terminal" :size="13" />
      </button>
      <button
        v-if="!hasGit"
        class="open-tool"
        type="button"
        :title="t('workArea.tabGit')"
        @click="openTab('git')"
      >
        <Icon name="git" :size="13" />
      </button>
      <button
        v-if="!hasLogs"
        class="open-tool"
        type="button"
        :title="t('workArea.tabLogs')"
        @click="openTab('logs')"
      >
        <Icon name="activity" :size="13" />
      </button>
      <button
        v-if="!hasChanges"
        class="open-tool"
        type="button"
        :title="t('workArea.tabChanges')"
        @click="openTab('changes')"
      >
        <Icon name="diff" :size="13" />
      </button>

      <span class="spacer"></span>

      <button
        class="follow"
        :class="{ on: settings.followCompanion }"
        type="button"
        :title="settings.followCompanion ? t('workArea.followOn') : t('workArea.followOff')"
        :aria-pressed="settings.followCompanion"
        @click="settings.followCompanion = !settings.followCompanion"
      >
        <Icon :name="settings.followCompanion ? 'eye' : 'eye-off'" :size="13" />
      </button>
    </div>

    <div class="content">
      <TerminalPanel
        v-if="hasTerminal"
        v-show="activeTab?.kind === 'terminal'"
        :active="activeTab?.kind === 'terminal'"
      />
      <template v-if="activeTab?.kind !== 'terminal'">
        <ProgressTab v-if="activeTab?.kind === 'progress'" />
        <ViewFileTab v-else-if="activeTab?.kind === 'file'" :path="activeTab.filePath ?? null" />
        <SkillsTab v-else-if="activeTab?.kind === 'skills'" />
        <SchedulesTab v-else-if="activeTab?.kind === 'schedules'" />
        <GitPanel v-else-if="activeTab?.kind === 'git'" />
        <LogsPanel v-else-if="activeTab?.kind === 'logs'" />
        <TurnDiffPanel v-else-if="activeTab?.kind === 'changes'" />
        <p v-else class="empty">{{ t("workArea.empty") }}</p>
      </template>
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

/* A file the companion just opened lands in the active tab. The glow says "this changed",
   then goes away — no dialog, no focus change. */
.tab.attention {
  animation: tab-attention 1.8s ease-out;
}

@keyframes tab-attention {
  0% {
    box-shadow: 0 0 0 2px transparent;
  }

  20% {
    box-shadow: 0 0 0 2px var(--accent);
  }

  100% {
    box-shadow: 0 0 0 2px transparent;
  }
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

.follow {
  display: grid;
  place-items: center;
  width: 24px;
  height: 24px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-muted);
}

.follow:hover {
  background: var(--bg-hover);
  color: var(--text-strong);
}

.follow.on {
  color: var(--accent);
}

.open-tool {
  display: grid;
  place-items: center;
  width: 24px;
  height: 24px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-muted);
}

.open-tool:hover {
  background: var(--bg-hover);
  color: var(--text-strong);
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
