<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import Icon from "./Icon.vue";
import { t } from "../i18n";

/**
 * The consent gate for the companion's full control. It is deliberately explicit: the
 * checkbox is required, the powers are listed, and the dialog says that undo is not a
 * sandbox. The same dialog is used by the composer and by Settings.
 */
const props = defineProps<{ open: boolean; name: string }>();
const emit = defineEmits<{ confirm: []; cancel: [] }>();

const confirmed = ref(false);
const checkbox = ref<HTMLInputElement | null>(null);

watch(
  () => props.open,
  (open) => {
    if (!open) return;
    confirmed.value = false;
    void nextTick(() => checkbox.value?.focus());
  },
);

function onKey(event: KeyboardEvent): void {
  if (!props.open || event.key !== "Escape") return;
  event.preventDefault();
  event.stopPropagation();
  emit("cancel");
}

onMounted(() => window.addEventListener("keydown", onKey, true));
onBeforeUnmount(() => window.removeEventListener("keydown", onKey, true));

function confirm(): void {
  if (!confirmed.value) return;
  emit("confirm");
}
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="rudder-backdrop" @click.self="emit('cancel')">
      <div
        class="rudder-dialog"
        role="dialog"
        aria-modal="true"
        :aria-label="t('rudder.title', { name })"
      >
        <div class="rudder-heading">
          <span class="rudder-mark"><Icon name="rudder" :size="20" /></span>
          <h2>{{ t("rudder.title", { name }) }}</h2>
        </div>

        <p class="rudder-warning">{{ t("rudder.warning") }}</p>
        <ul class="rudder-powers">
          <li>{{ t("rudder.powerFiles") }}</li>
          <li>{{ t("rudder.powerCommands") }}</li>
          <li>{{ t("rudder.powerNetwork") }}</li>
        </ul>
        <p class="rudder-note">{{ t("rudder.noSandbox") }}</p>

        <label class="rudder-check">
          <input ref="checkbox" v-model="confirmed" type="checkbox" />
          <span>{{ t("rudder.confirmCheckbox") }}</span>
        </label>

        <div class="rudder-actions">
          <button class="mini-text" type="button" @click="emit('cancel')">
            {{ t("rudder.cancel") }}
          </button>
          <button class="rudder-confirm" type="button" :disabled="!confirmed" @click="confirm">
            {{ t("rudder.confirm") }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.rudder-backdrop {
  position: fixed;
  inset: 0;
  z-index: 3000;
  display: grid;
  place-items: center;
  padding: 16px;
  background: rgba(0, 0, 0, 0.62);
}

.rudder-dialog {
  width: min(470px, 100%);
  padding: 20px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-lg);
  background: var(--bg-elevated);
  box-shadow: 0 24px 70px rgba(0, 0, 0, 0.55);
  color: var(--text);
}

.rudder-heading {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
}

.rudder-mark {
  display: grid;
  place-items: center;
  width: 34px;
  height: 34px;
  border: 1px solid var(--border-strong);
  border-radius: 50%;
  color: var(--accent);
}

.rudder-heading h2 {
  margin: 0;
  color: var(--text-strong);
  font-size: 16px;
  font-weight: 600;
}

.rudder-warning {
  margin: 0 0 8px;
  color: var(--text-strong);
  font-size: 13px;
}

.rudder-powers {
  margin: 0 0 10px;
  padding-left: 20px;
  color: var(--text);
  font-size: 12px;
  line-height: 1.7;
}

.rudder-note {
  margin: 0 0 14px;
  padding: 8px 10px;
  border-left: 2px solid var(--danger);
  background: var(--bg-panel);
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.5;
}

.rudder-check {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  margin-bottom: 16px;
  color: var(--text);
  font-size: 12px;
  line-height: 1.5;
  cursor: pointer;
}

.rudder-check input {
  margin-top: 2px;
  accent-color: var(--accent);
}

.rudder-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.rudder-confirm {
  height: 30px;
  padding: 0 12px;
  border: 1px solid var(--accent);
  border-radius: var(--radius-sm);
  background: var(--accent);
  color: var(--bg-app);
  font-size: 12px;
  font-weight: 600;
}

.rudder-confirm:disabled {
  cursor: default;
  opacity: 0.45;
}
</style>
