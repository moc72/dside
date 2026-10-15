<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import Icon from "./Icon.vue";
import { t } from "../i18n";
import { links } from "../engine/links";
import type { Session, SessionGroup } from "../types";
import { activeSessionId, sessions } from "../stores/sessions";
import { settings } from "../stores/settings";
import { openSearch, openSettings, railCollapsed } from "../stores/ui";
import { openTab } from "../stores/workArea";
import {
  clearApiKey,
  createSession,
  engineReady,
  loadSessions,
  openSession,
} from "../composables/useEngine";

const accountOpen = ref(false);
const systemName = ref("User");
const displayName = computed(() => settings.displayName.trim() || systemName.value);

onMounted(async () => {
  if (settings.displayName.trim()) return;
  try {
    systemName.value = await invoke<string>("default_display_name");
  } catch {
    // keep the fallback
  }
});

async function openLink(url: string) {
  accountOpen.value = false;
  try {
    await openUrl(url);
  } catch {
    // no browser available
  }
}

async function signOut() {
  accountOpen.value = false;
  await clearApiKey();
}

function signIn() {
  accountOpen.value = false;
  openSettings("usage");
}
const menuSession = ref<Session | null>(null);
const menuX = ref(0);
const menuY = ref(0);
const renamingId = ref<string | null>(null);
const renameDraft = ref("");

const vFocus = {
  mounted: (element: HTMLInputElement) => element.focus(),
};

function openMenu(event: MouseEvent, session: Session) {
  event.preventDefault();
  menuX.value = event.clientX;
  menuY.value = event.clientY;
  menuSession.value = session;
}

function startRename(session: Session) {
  menuSession.value = null;
  renamingId.value = session.id;
  renameDraft.value = session.title;
}

async function commitRename() {
  const id = renamingId.value;
  const title = renameDraft.value.trim();
  renamingId.value = null;
  if (!id || !title) return;
  await invoke("session_update", { id, title });
  await loadSessions();
}

async function togglePin(session: Session) {
  menuSession.value = null;
  await invoke("session_update", {
    id: session.id,
    group: session.group === "pinned" ? "project" : "pinned",
  });
  await loadSessions();
}

/** Hides the session from the rail. Nothing is deleted — see the archive in Settings. */
async function archiveSession(session: Session) {
  menuSession.value = null;
  await invoke("session_archive", { id: session.id, archived: true });
  await loadSessions();
}

const primaryItems = [
  { icon: "plus", key: "nav.newTask", soon: false },
  { icon: "search", key: "nav.search", soon: false },
  { icon: "grid", key: "nav.skills", soon: false },
  { icon: "clock", key: "nav.scheduled", soon: false },
  { icon: "globe", key: "nav.site", soon: true },
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
  if (key === "nav.newTask") {
    void createSession();
    return;
  }
  if (key === "nav.search") {
    openSearch();
    return;
  }
  if (key === "nav.skills") {
    openTab("skills");
    return;
  }
  if (key === "nav.scheduled") {
    openTab("schedules");
  }
}

function accountSettings() {
  accountOpen.value = false;
  openSettings();
}
</script>

<template>
  <aside class="rail" :class="{ collapsed: railCollapsed }">
    <div v-if="menuSession" class="menu-backdrop" @click="menuSession = null" @contextmenu.prevent="menuSession = null"></div>
    <div
      v-if="menuSession"
      class="session-menu"
      :style="{ left: `${menuX}px`, top: `${menuY}px` }"
    >
      <button class="menu-item" type="button" @click="togglePin(menuSession)">
        <Icon name="pin" :size="14" />
        <span>{{ menuSession.group === "pinned" ? t("session.unpin") : t("session.pin") }}</span>
      </button>
      <button class="menu-item" type="button" @click="startRename(menuSession)">
        <Icon name="pencil" :size="14" />
        <span>{{ t("session.rename") }}</span>
      </button>
      <button class="menu-item" type="button" @click="archiveSession(menuSession)">
        <Icon name="archive" :size="14" />
        <span>{{ t("session.archive") }}</span>
      </button>
    </div>
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
        <span v-if="item.soon" class="soon">{{ t("nav.soon") }}</span>
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
          @contextmenu.prevent="openMenu($event, session)"
        >
          <Icon name="doc" :size="14" />
          <input
            v-if="renamingId === session.id"
            v-model="renameDraft"
            v-focus
            class="rename"
            @click.stop
            @keydown.enter.prevent="commitRename"
            @keydown.esc.prevent="renamingId = null"
            @blur="commitRename"
          />
          <span v-else>{{ session.title }}</span>
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
        <div class="menu-head">
          <span class="menu-name">{{ displayName }}</span>
          <span class="menu-provider">{{ t("account.provider") }}</span>
        </div>
        <button class="menu-item" type="button" @click="accountSettings">
          <Icon name="gear" :size="14" /><span>{{ t("account.settings") }}</span>
        </button>
        <button class="menu-item" type="button" @click="openLink(links.provider)">
          <Icon name="user" :size="14" /><span>{{ t("account.account") }}</span>
          <Icon name="external-link" :size="14" class="trail" />
        </button>
        <button class="menu-item" type="button" @click="openLink(links.usage)">
          <Icon name="grid" :size="14" /><span>{{ t("account.usage") }}</span>
          <Icon name="external-link" :size="14" class="trail" />
        </button>
        <button class="menu-item" type="button" @click="openLink(links.github)">
          <Icon name="send" :size="14" /><span>{{ t("account.feedback") }}</span>
          <Icon name="external-link" :size="14" class="trail" />
        </button>
        <button class="menu-item" type="button" @click="openLink(links.issues)">
          <Icon name="doc" :size="14" /><span>{{ t("account.contact") }}</span>
          <Icon name="external-link" :size="14" class="trail" />
        </button>
        <button class="menu-item" type="button" @click="openLink(links.home)">
          <Icon name="globe" :size="14" /><span>{{ t("account.learnMore") }}</span>
          <Icon name="external-link" :size="14" class="trail" />
        </button>
        <button v-if="engineReady" class="menu-item" type="button" @click="signOut">
          <Icon name="external-link" :size="14" /><span>{{ t("account.logout") }}</span>
        </button>
        <button v-else class="menu-item accent" type="button" @click="signIn">
          <Icon name="sparkle" :size="14" /><span>{{ t("account.signIn") }}</span>
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

.soon {
  margin-left: auto;
  padding: 0 6px;
  border: 1px solid var(--border-strong);
  border-radius: 999px;
  color: var(--text-muted);
  font-size: 10px;
  line-height: 15px;
}

.rename {
  flex: 1;
  min-width: 0;
  height: 20px;
  padding: 0 4px;
  border: 1px solid var(--accent);
  border-radius: 3px;
  background: var(--bg-input);
  color: var(--text-strong);
  font-size: 12px;
}

.menu-backdrop {
  position: fixed;
  inset: 0;
  z-index: 41;
}

.session-menu {
  position: fixed;
  z-index: 42;
  min-width: 180px;
  padding: 6px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius);
  background: var(--bg-elevated);
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.55);
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

.menu-head {
  display: flex;
  align-items: baseline;
  gap: 8px;
  padding: 6px 10px 10px;
}

.menu-name {
  color: var(--text-strong);
  font-size: 13px;
  font-weight: 600;
}

.menu-provider {
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
