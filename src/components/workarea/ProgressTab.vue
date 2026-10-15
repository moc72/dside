<script setup lang="ts">
import { ref } from "vue";
import Icon from "../Icon.vue";
import { t } from "../../i18n";
import { deliverablesByGroup } from "../../stores/sessions";

const collapsed = ref<Record<string, boolean>>({});

function toggle(group: string) {
  collapsed.value[group] = !collapsed.value[group];
}
</script>

<template>
  <div class="progress">
    <h2 class="title">{{ t("progress.title") }}</h2>
    <p class="subtitle">{{ t("progress.subtitle") }}</p>

    <div class="section-head">
      <span>{{ t("progress.deliverables") }}</span>
      <Icon name="chevron-down" :size="14" />
    </div>

    <p v-if="deliverablesByGroup.length === 0" class="empty">{{ t("progress.empty") }}</p>

    <div v-for="group in deliverablesByGroup" :key="group.group" class="group">
      <button class="group-head" type="button" @click="toggle(group.group)">
        <span>{{ group.group }}</span>
        <Icon :name="collapsed[group.group] ? 'chevron-right' : 'chevron-down'" :size="13" />
      </button>
      <ul v-show="!collapsed[group.group]" class="items">
        <li v-for="item in group.items" :key="item.id">
          <Icon name="doc" :size="13" />
          <span>{{ item.label }}</span>
        </li>
      </ul>
    </div>
  </div>
</template>

<style scoped>
.progress {
  padding: 16px 14px;
  overflow-y: auto;
  height: 100%;
}

.title {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--text-strong);
}

.subtitle {
  margin: 4px 0 16px;
  color: var(--text-muted);
  font-size: 12px;
}

.section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 0;
  border-top: 1px solid var(--border);
  color: var(--text-strong);
  font-size: 12px;
  font-weight: 600;
}

.group {
  margin-top: 4px;
}

.group-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 5px 2px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  font-size: 12px;
}

.group-head:hover {
  color: var(--text);
}

.items {
  list-style: none;
  margin: 0;
  padding: 0;
}

.items li {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 2px;
  color: var(--text);
  font-size: 12px;
}

.items li span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.empty {
  color: var(--text-muted);
  font-size: 12px;
}
</style>
