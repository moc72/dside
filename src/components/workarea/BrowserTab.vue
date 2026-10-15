<script setup lang="ts">
import { ref } from "vue";
import Icon from "../Icon.vue";
import { t } from "../../i18n";

const url = ref("");
</script>

<template>
  <div class="browser">
    <div class="bar">
      <button class="icon" type="button" :title="t('browser.back')">
        <Icon name="arrow-left" :size="14" />
      </button>
      <button class="icon" type="button" :title="t('browser.forward')">
        <Icon name="arrow-right" :size="14" />
      </button>
      <button class="icon" type="button" :title="t('browser.reload')">
        <Icon name="reload" :size="14" />
      </button>
      <input v-model="url" class="url" :placeholder="t('browser.placeholder')" />
      <button class="icon" type="button" :title="t('browser.menu')">
        <Icon name="dots" :size="14" />
      </button>
    </div>

    <div class="empty">
      <Icon name="globe" :size="30" />
      <p>{{ t("browser.noServers") }}</p>
    </div>
  </div>
</template>

<style scoped>
.browser {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.bar {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 8px 10px;
  border-bottom: 1px solid var(--border);
}

.icon {
  display: grid;
  place-items: center;
  width: 26px;
  height: 26px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-muted);
}

.icon:hover {
  background: var(--bg-hover);
  color: var(--text-strong);
}

.url {
  flex: 1;
  min-width: 0;
  height: 28px;
  padding: 0 12px;
  border: 1px solid var(--border);
  border-radius: 14px;
  background: var(--bg-input);
  color: var(--text);
  font-size: 12px;
}

.url:focus {
  outline: none;
  border-color: var(--accent);
}

.empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  color: var(--text-muted);
}
</style>
