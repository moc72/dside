<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Icon from "../Icon.vue";
import { t } from "../../i18n";
import { activeSessionId } from "../../stores/sessions";
import { todosRevision } from "../../composables/useEngine";

interface TodoRow {
  id: number;
  content: string;
  status: string | null;
  priority: string | null;
}

const rows = ref<TodoRow[]>([]);
const loading = ref(false);

const statuses = ["in_progress", "pending", "completed"] as const;

const statusLabels: Record<string, "progress.statusInProgress" | "progress.statusPending" | "progress.statusCompleted"> = {
  in_progress: "progress.statusInProgress",
  pending: "progress.statusPending",
  completed: "progress.statusCompleted",
};

const grouped = computed(() =>
  statuses
    .map((status) => ({
      status,
      items: rows.value.filter((row) => (row.status ?? "pending") === status),
    }))
    .filter((group) => group.items.length > 0),
);

async function load() {
  loading.value = true;
  try {
    rows.value = await invoke<TodoRow[]>("session_todos", {
      sessionId: activeSessionId.value,
    });
  } catch {
    rows.value = [];
  } finally {
    loading.value = false;
  }
}

watch([activeSessionId, todosRevision], () => void load(), { immediate: true });
</script>

<template>
  <div class="progress">
    <h2 class="title">{{ t("progress.title") }}</h2>
    <p class="subtitle">{{ t("progress.subtitle") }}</p>

    <div class="section-head">
      <span>{{ t("progress.tasks") }}</span>
      <Icon name="chevron-down" :size="14" />
    </div>

    <p v-if="loading" class="empty">{{ t("viewFile.loading") }}</p>
    <p v-else-if="grouped.length === 0" class="empty">{{ t("progress.empty") }}</p>

    <div v-for="group in grouped" :key="group.status" class="group">
      <div class="group-head">
        <span>{{ t(statusLabels[group.status]) }}</span>
        <span class="count">{{ group.items.length }}</span>
      </div>
      <ul class="items">
        <li v-for="item in group.items" :key="item.id" :class="group.status">
          <Icon
            :name="group.status === 'completed' ? 'check' : group.status === 'in_progress' ? 'reload' : 'doc'"
            :size="13"
          />
          <span>{{ item.content }}</span>
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
  margin-top: 10px;
}

.group-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 5px 2px;
  color: var(--text-muted);
  font-size: 12px;
  font-weight: 600;
}

.count {
  font-weight: 400;
  opacity: 0.7;
}

.items {
  list-style: none;
  margin: 0;
  padding: 0;
}

.items li {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 5px 2px;
  color: var(--text);
  font-size: 12px;
  line-height: 1.45;
}

.items li.completed {
  color: var(--text-muted);
  text-decoration: line-through;
}

.empty {
  color: var(--text-muted);
  font-size: 12px;
}
</style>
