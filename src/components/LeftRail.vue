<script setup lang="ts">
import { ref } from "vue";
import Icon from "./Icon.vue";
import { t } from "../i18n";
import type { SessionGroup } from "../types";
import { activeSessionId, sessions } from "../stores/sessions";
import { openSettings, railCollapsed } from "../stores/ui";
import { createSession, openSession } from "../composables/useEngine";

const accountOpen = ref(false);

const primaryItems = [
  { icon: "plus", key: "nav.newTask" },
  { icon: "search", key: "nav.search" },
  { icon: "grid", key: "nav.skills" },
  { icon: "clock", key: "nav.scheduled" },
  { icon: "globe", key: "nav.site" },
] as const;

const sectionOrder: SessionGroup[] = ["pinned", "scheduled", "project", "team"];

const sectionKeys: Record<SessionGroup, "sections.pinned" | "sections.scheduled" | "sections.projects" | "sections.agentTeam"> = {
  pinned: "sections.pinned",
  scheduled: "sections.scheduled",
  project: "sections.projects",
  team: "sections.agentTeam",
};

function groupItems(group: SessionGroup) {
  return sessions.value.filter((session) => session.group === group);
}

function onPrimary(key: string) {
  if (key === "nav.newTask") void createSession();
}

function accountSettings() {
  accountOpen.value = false;
  openSettings();
}
</script>

<template>
  <aside class="rail" :class="{ collapsed: railCollapsed }">
    <div class="rail-head">
      <span class="brand">{{ t("app.name") }}</span>
    </div>

    <nav class="rail-nav">
      <button
        v-for="item in primaryItems"
        :key="item.key"
        class="nav-item"
        type="button"
        @click="onPrimary(item.key)"
      >
        <Icon :name="item.icon" />
        <span>{{ t(item.key) }}</span>
      </button>
    </nav>

    <div class="rail-scroll">
      <section v-for="group in sectionOrder" :key="group" class="rail-section">
        <div class="section-head">{{ t(sectionKeys[group]) }}</div>
        <p v-if="groupItems(group).length === 0" class="empty">{{ t("sections.noTasks") }}</p>
        <button
          v-for="session in groupItems(group)"
          :key="session.id"
          class="session"
          :class="{ active: session.id === activeSessionId }"
          type="button"
          @click="openSession(session.id)"
        >
          <Icon name="doc" :size="14" />
          <span>{{ session.title }}</span>
        </button>
      </section>
    </div>

    <div class="rail-foot">
      <button class="account" type="button" @click.stop="accountOpen = !accountOpen">
        <span class="avatar"><Icon name="sparkle" :size="14" /></span>
        <span class="account-name">modest</span>
        <Icon name="chevron-down" :size="14" />
      </button>

      <div v-if="accountOpen" class="backdrop" @click="accountOpen = false"></div>
      <div v-if="accountOpen" class="account-menu">
        <div class="menu-uid">{{ t("account.uid") }}: 539944927909126149</div>
        <button class="menu-item" type="button" @click="accountSettings">
          <Icon name="gear" :size="14" /><span>{{ t("account.settings") }}</span>
        </button>
        <button class="menu-item" type="button">
          <Icon name="grid" :size="14" /><span>{{ t("account.usage") }}</span>
          <Icon name="chevron-right" :size="14" class="trail" />
        </button>
        <button class="menu-item" type="button">
          <Icon name="send" :size="14" /><span>{{ t("account.feedback") }}</span>
          <Icon name="chevron-right" :size="14" class="trail" />
        </button>
        <button class="menu-item" type="button">
          <Icon name="doc" :size="14" /><span>{{ t("account.changelog") }}</span>
        </button>
        <button class="menu-item" type="button">
          <Icon name="user" :size="14" /><span>{{ t("account.contact") }}</span>
          <Icon name="chevron-right" :size="14" class="trail" />
        </button>
        <button class="menu-item" type="button">
          <Icon name="external-link" :size="14" /><span>{{ t("account.learnMore") }}</span>
          <Icon name="chevron-right" :size="14" class="trail" />
        </button>
        <button class="menu-item" type="button">
          <Icon name="external-link" :size="14" /><span>{{ t("account.logout") }}</span>
        </button>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.rail {
  position: relative;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: visible;
}

.rail-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 44px;
  padding: 0 8px 0 14px;
}

.brand {
  font-weight: 700;
  color: var(--text-strong);
  white-space: nowrap;
}

.rail-nav {
  display: flex;
  flex-direction: column;
  padding: 4px 8px 8px;
}

.nav-item,
.session,
.menu-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 7px 10px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text);
  text-align: left;
  white-space: nowrap;
}

.nav-item:hover,
.session:hover,
.menu-item:hover {
  background: var(--bg-hover);
}

.session.active {
  background: var(--bg-active);
  color: var(--text-strong);
}

.rail-scroll {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 4px 8px 12px;
}

.rail-section {
  margin-top: 12px;
}

.section-head {
  padding: 4px 10px;
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.6px;
  text-transform: uppercase;
}

.empty {
  margin: 2px 0 0;
  padding: 4px 10px;
  color: var(--text-muted);
  font-size: 12px;
}

.rail-foot {
  position: relative;
  padding: 8px;
  border-top: 1px solid var(--border);
}

.account {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 6px 10px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-strong);
}

.account:hover {
  background: var(--bg-hover);
}

.avatar {
  display: grid;
  place-items: center;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: var(--accent);
  color: var(--accent-contrast);
}

.account-name {
  flex: 1;
  text-align: left;
}

.backdrop {
  position: fixed;
  inset: 0;
  z-index: 39;
}

.account-menu {
  position: absolute;
  bottom: 52px;
  left: 8px;
  z-index: 40;
  width: min(260px, calc(100vw - 24px));
  padding: 6px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius);
  background: var(--bg-elevated);
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.55);
}

.menu-uid {
  padding: 6px 10px 10px;
  color: var(--text-muted);
  font-size: 11px;
}

.menu-item.accent {
  color: var(--accent);
}

.trail {
  margin-left: auto;
  color: var(--text-muted);
}
</style>
