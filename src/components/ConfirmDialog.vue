<script setup lang="ts">
import { nextTick, ref, watch } from "vue";

/**
 * One question, two answers. A native `window.confirm` looks like a browser warning and
 * cannot be styled; this keeps the app's own voice for a decision that changes a file.
 * The safe button gets the focus, so Enter never confirms by accident.
 */
const props = defineProps<{
  open: boolean;
  title: string;
  message: string;
  confirmLabel: string;
  cancelLabel: string;
  danger?: boolean;
}>();

const emit = defineEmits<{ confirm: []; cancel: [] }>();

const cancelButton = ref<HTMLButtonElement | null>(null);

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) void nextTick(() => cancelButton.value?.focus());
  },
);

function onKeydown(event: KeyboardEvent): void {
  if (event.key === "Escape") {
    event.preventDefault();
    emit("cancel");
  }
}
</script>

<template>
  <div v-if="open" class="backdrop" @click.self="emit('cancel')" @keydown="onKeydown">
    <div class="dialog" role="dialog" aria-modal="true" :aria-label="title">
      <h2>{{ title }}</h2>
      <p>{{ message }}</p>
      <div class="actions">
        <button ref="cancelButton" class="cancel" type="button" @click="emit('cancel')">
          {{ cancelLabel }}
        </button>
        <button
          class="confirm"
          :class="{ danger }"
          type="button"
          @click="emit('confirm')"
        >
          {{ confirmLabel }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.backdrop {
  position: fixed;
  inset: 0;
  z-index: 80;
  display: grid;
  place-items: center;
  padding: 24px;
  background: rgb(0 0 0 / 55%);
  backdrop-filter: blur(2px);
}

.dialog {
  width: min(440px, 100%);
  padding: 20px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius);
  background: var(--bg-app);
  box-shadow: 0 24px 70px rgb(0 0 0 / 55%);
}

h2 {
  margin: 0 0 8px;
  color: var(--text-strong);
  font-size: 16px;
}

p {
  margin: 0 0 18px;
  color: var(--text);
  font-size: 13px;
  line-height: 1.55;
}

.actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

button {
  height: 30px;
  padding: 0 14px;
  border-radius: var(--radius-sm);
  font-size: 12px;
}

.cancel {
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text);
}

.cancel:hover {
  background: var(--bg-hover);
}

.confirm {
  border: 1px solid var(--accent);
  background: var(--accent);
  color: var(--accent-contrast);
  font-weight: 600;
}

.confirm:hover {
  filter: brightness(1.08);
}

.confirm.danger {
  border-color: var(--danger);
  background: var(--danger);
  color: #fff;
}
</style>
