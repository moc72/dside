<script setup lang="ts">
import { computed, watch } from "vue";
import Icon from "../Icon.vue";
import { t } from "../../i18n";
import type { WorkAreaTabKind } from "../../types";
import { activeTab, activateTab, attentionTabId, closeTab, tabs } from "../../stores/workArea";
import { settings } from "../../stores/settings";
import ProgressTab from "./ProgressTab.vue";
import ViewFileTab from "./ViewFileTab.vue";
import SkillsTab from "./SkillsTab.vue";
import SchedulesTab from "./SchedulesTab.vue";
import { EVERYDAY_HIDDEN_TABS, everydayMode } from "../../engine/mode";

const labels: Record<
  WorkAreaTabKind,
  | "workArea.tabProgress"
  | "workArea.tabFile"
  | "workArea.tabSkills"
  | "workArea.tabSchedules"
> = {
  progress: "workArea.tabProgress",
  file: "workArea.tabFile",
  skills: "workArea.tabSkills",
  schedules: "workArea.tabSchedules",
};

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
      <ProgressTab v-if="activeTab?.kind === 'progress'" />
      <ViewFileTab v-else-if="activeTab?.kind === 'file'" :path="activeTab.filePath ?? null" />
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
