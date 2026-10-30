<script setup lang="ts">
import { computed, defineAsyncComponent, onBeforeUnmount, onMounted, ref, watch } from "vue";
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

/**
 * The panels that are not open yet, offered behind one button instead of four loose icons.
 * Their icons are recognisable, but four of them sat between the tabs and the panel toggle
 * and read as tabs of their own. One button with a list keeps the bar calm and still reachable.
 */
const toolMenuOpen = ref(false);

const closedTools = computed(() =>
  [
    { kind: "terminal", label: "workArea.tabTerminal", icon: "terminal" },
    { kind: "git", label: "workArea.tabGit", icon: "git" },
    { kind: "logs", label: "workArea.tabLogs", icon: "activity" },
    { kind: "changes", label: "workArea.tabChanges", icon: "diff" },
  ].filter((tool) => !tabs.value.some((tab) => tab.kind === tool.kind)),
);

function openTool(kind: string): void {
  toolMenuOpen.value = false;
  openTab(kind as never);
}

function onToolMenuKey(event: KeyboardEvent): void {
  if (event.key === "Escape") toolMenuOpen.value = false;
}

onMounted(() => window.addEventListener("keydown", onToolMenuKey));
onBeforeUnmount(() => window.removeEventListener("keydown", onToolMenuKey));

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

      <span class="spacer"></span>

      <!-- One button for "open another panel", directly before the panel toggle. -->
      <div v-if="closedTools.length" class="tool-menu">
        <button
          class="open-tool"
          type="button"
          :title="t('workArea.openPanel')"
          :aria-label="t('workArea.openPanel')"
          :aria-expanded="toolMenuOpen"
          @click="toolMenuOpen = !toolMenuOpen"
        >
          <Icon name="grid" :size="13" />
        </button>

        <div v-if="toolMenuOpen" class="backdrop" @click="toolMenuOpen = false"></div>
        <div v-if="toolMenuOpen" class="tool-list">
          <button
            v-for="tool in closedTools"
            :key="tool.kind"
            class="tool-item"
            type="button"
            @click="openTool(tool.kind)"
          >
            <Icon :name="tool.icon" :size="13" />
            <span>{{ t(tool.label as never) }}</span>
          </button>
        </div>
      </div>

      <button
        class="follow"
        :class="{ on: settings.followCompanion }"
        type="button"
        :title="settings.followCompanion ? t('workArea.followOn') : t('workArea.followOff')"
        :aria-pressed="settings.followCompanion"
        @click="settings.followCompanion = !settings.followCompanion"
      >
        <Icon :name="settings.followCompanion ? 'robot' : 'robot-off'" :size="14" />
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

.tool-menu {
  position: relative;
  display: flex;
  align-items: center;
}

.tool-menu .backdrop {
  position: fixed;
  inset: 0;
  z-index: 20;
}

.tool-list {
  position: absolute;
  top: 28px;
  right: 0;
  z-index: 21;
  min-width: 150px;
  padding: 4px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  background: var(--bg-panel);
  box-shadow: 0 14px 40px rgb(0 0 0 / 45%);
}

.tool-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 6px 9px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text);
  font-size: 12px;
  text-align: left;
}

.tool-item:hover {
  background: var(--bg-hover);
  color: var(--text-strong);
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
