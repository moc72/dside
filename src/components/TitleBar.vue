<script setup lang="ts">
import Icon from "./Icon.vue";
import ToolbarActions from "./ToolbarActions.vue";
import { t } from "../i18n";
import { activeSession } from "../stores/sessions";
import { toggleRail } from "../stores/ui";
</script>

<template>
  <header class="titlebar">
    <div class="side left">
      <button class="ghost" type="button" :title="t('nav.collapseSidebar')" @click="toggleRail">
        <Icon name="panel-left" />
      </button>
    </div>

    <div class="center">
      <span class="session">{{ activeSession?.title ?? "" }}</span>
      <Icon name="chevron-down" :size="14" />
    </div>

    <div class="side right">
      <ToolbarActions />
    </div>
  </header>
</template>

<style scoped>
.titlebar {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: center;
  height: 44px;
  padding: 0 14px;
  background: transparent;
}

.side {
  display: flex;
  align-items: center;
  gap: 6px;
}

.right {
  justify-content: flex-end;
}

.center {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--text-strong);
  font-size: 14px;
  font-weight: 600;
  white-space: nowrap;
}

.ghost {
  display: grid;
  place-items: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-muted);
}

.ghost:hover {
  background: var(--bg-hover);
  color: var(--text-strong);
}
</style>
