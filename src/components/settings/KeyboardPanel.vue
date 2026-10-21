<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from "vue";
import Icon from "../Icon.vue";
import { t } from "../../i18n";
import {
  activeBindings,
  bindingFor,
  bindingOwner,
  COMMANDS,
  isSafeGlobalBinding,
  resetAllBindings,
  resetCommand,
  setBindings,
  shortcutCapturing,
} from "../../engine/shortcuts";
import { settings } from "../../stores/settings";
import type { ShortcutCommand } from "../../engine/shortcuts";

const query = ref("");
const editing = ref<ShortcutCommand | null>(null);
const replacing = ref<string | null>(null);
const captured = ref<string | null>(null);
const captureError = ref<string | null>(null);

const filtered = computed(() => {
  const needle = query.value.trim().toLowerCase();
  if (!needle) return COMMANDS;
  return COMMANDS.filter(
    (command) =>
      t(command.label).toLowerCase().includes(needle) ||
      activeBindings(command.id).some((binding) => binding.toLowerCase().includes(needle)),
  );
});

const hasOverrides = computed(() => Object.keys(settings.shortcuts ?? {}).length > 0);

function parts(binding: string | null): string[] {
  return binding ? binding.split("+") : [];
}

function keyParts(binding: string): string[] {
  return parts(binding);
}

function isCustom(id: string): boolean {
  return Array.isArray(settings.shortcuts?.[id]);
}

function startCapture(command: ShortcutCommand, replace: string | null): void {
  if (command.fixed) return;
  editing.value = command;
  replacing.value = replace;
  captured.value = null;
  captureError.value = null;
  shortcutCapturing.value = true;
  void nextTick(() => {
    // The dialog listens on the window; focus just keeps the visual focus inside it.
    document.querySelector<HTMLElement>(".shortcut-modal")?.focus();
  });
}

function closeCapture(): void {
  editing.value = null;
  replacing.value = null;
  captured.value = null;
  captureError.value = null;
  shortcutCapturing.value = false;
}

function onKey(event: KeyboardEvent): void {
  if (!editing.value) return;
  event.preventDefault();
  event.stopPropagation();
  if (event.repeat) return;

  if (event.key === "Escape") {
    closeCapture();
    return;
  }
  if (event.key === "Backspace" || event.key === "Delete") {
    captured.value = null;
    captureError.value = null;
    return;
  }

  const binding = bindingFor(event);
  if (!binding) return;
  captured.value = binding;
  captureError.value = isSafeGlobalBinding(binding)
    ? null
    : t("shortcuts.needsModifier");
}

function saveCapture(): void {
  const command = editing.value;
  const value = captured.value;
  if (!command || !value || captureError.value) return;

  const owner = bindingOwner(value, command.id);
  if (owner) {
    captureError.value = t("shortcuts.conflict", { command: t(owner.label) });
    return;
  }

  const bindings = activeBindings(command.id).filter((binding) => binding !== replacing.value);
  if (!bindings.includes(value)) bindings.push(value);
  setBindings(command.id, bindings);
  closeCapture();
}

function removeBinding(command: ShortcutCommand, binding: string): void {
  if (command.fixed) return;
  setBindings(
    command.id,
    activeBindings(command.id).filter((entry) => entry !== binding),
  );
}

function resetOne(command: ShortcutCommand): void {
  resetCommand(command.id);
}

function resetAll(): void {
  resetAllBindings();
}

onMounted(() => window.addEventListener("keydown", onKey, true));
onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKey, true);
  shortcutCapturing.value = false;
});
</script>

<template>
  <section class="shortcuts">
    <p class="hint">{{ t("shortcuts.hint") }}</p>

    <div class="toolbar">
      <label class="search">
        <Icon name="search" :size="14" />
        <input
          v-model="query"
          type="search"
          :placeholder="t('shortcuts.search')"
          :aria-label="t('shortcuts.search')"
        />
      </label>
      <button
        class="mini-text"
        type="button"
        :disabled="!hasOverrides"
        @click="resetAll"
      >
        <Icon name="reload" :size="13" />
        {{ t("shortcuts.resetAll") }}
      </button>
    </div>

    <div class="table">
      <div class="row table-head">
        <span>{{ t("shortcuts.command") }}</span>
        <span>{{ t("shortcuts.binding") }}</span>
        <span>{{ t("shortcuts.scope") }}</span>
        <span></span>
      </div>

      <div v-for="command in filtered" :key="command.id" class="row">
        <span class="label">{{ t(command.label) }}</span>

        <span class="keys">
          <span v-for="binding in activeBindings(command.id)" :key="binding" class="key">
            <button
              class="key-label"
              type="button"
              :title="command.fixed ? t('shortcuts.fixed') : t('shortcuts.press')"
              :disabled="command.fixed"
              @click="startCapture(command, binding)"
            >
              <kbd v-for="part in keyParts(binding)" :key="part">{{ part }}</kbd>
            </button>
            <button
              v-if="!command.fixed"
              class="key-remove"
              type="button"
              :title="t('shortcuts.remove')"
              @click="removeBinding(command, binding)"
            >
              <Icon name="x" :size="11" />
            </button>
          </span>

          <button
            v-if="!command.fixed"
            class="add"
            type="button"
            :title="t('shortcuts.add')"
            @click="startCapture(command, null)"
          >
            <Icon name="plus" :size="11" />
          </button>
          <span v-if="activeBindings(command.id).length === 0 && !command.fixed" class="none">
            {{ t("shortcuts.none") }}
          </span>
        </span>

        <span class="scope">
          {{ command.scope === "input" ? t("shortcuts.scopeInput") : t("shortcuts.scopeGlobal") }}
        </span>

        <span class="actions">
          <button
            v-if="!command.fixed && isCustom(command.id)"
            class="mini"
            type="button"
            :title="t('shortcuts.reset')"
            @click="resetOne(command)"
          >
            <Icon name="reload" :size="12" />
          </button>
        </span>
      </div>

      <p v-if="filtered.length === 0" class="empty">{{ t("shortcuts.empty") }}</p>
    </div>

    <Teleport to="body">
      <div v-if="editing" class="backdrop" @click.self="closeCapture">
        <div
          class="shortcut-modal"
          role="dialog"
          aria-modal="true"
          tabindex="-1"
          @keydown.stop
        >
          <h3>{{ t(editing.label) }}</h3>
          <div class="capture" :class="{ waiting: !captured }">
            <kbd v-for="part in parts(captured)" :key="part">{{ part }}</kbd>
            <span v-if="!captured" class="waiting-text">{{ t("shortcuts.press") }}</span>
          </div>
          <p class="hint">{{ t("shortcuts.pressHint") }}</p>
          <p v-if="captureError" class="error">{{ captureError }}</p>
          <div class="modal-actions">
            <button class="mini-text" type="button" @click="closeCapture">
              {{ t("shortcuts.cancel") }}
            </button>
            <button
              class="mini-text primary"
              type="button"
              :disabled="!captured || !!captureError"
              @click="saveCapture"
            >
              {{ t("shortcuts.save") }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </section>
</template>

<style scoped>
.shortcuts {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-width: 0;
}


.mini {
  display: grid;
  place-items: center;
  width: 26px;
  height: 26px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-input);
  color: var(--text-muted);
  cursor: pointer;
}

.mini:hover {
  color: var(--text-strong);
  border-color: var(--accent);
}

.mini-text {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 30px;
  padding: 0 11px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text);
  font-family: inherit;
  font-size: 12px;
  cursor: pointer;
}

.mini-text:hover {
  border-color: var(--accent);
  color: var(--text-strong);
}

.mini-text:disabled {
  opacity: 0.45;
  cursor: default;
}

.mini-text.primary {
  border-color: var(--accent);
  background: var(--accent);
  color: #fff;
}

.mini-text.primary:hover {
  filter: brightness(1.08);
  color: #fff;
}

.hint {
  margin: 0;
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.5;
}

.toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
}

.search {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
  height: 30px;
  padding: 0 9px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  background: var(--bg);
  color: var(--text-muted);
}

.search input {
  flex: 1;
  min-width: 0;
  border: none;
  background: transparent;
  color: var(--text);
  font-family: inherit;
  font-size: 12px;
}

.search input:focus {
  outline: none;
}

.search:focus-within {
  border-color: var(--accent);
}

.table {
  border: 1px solid var(--border);
  border-radius: var(--radius);
  overflow: hidden;
}

.row {
  display: grid;
  grid-template-columns: minmax(170px, 1.4fr) minmax(170px, 1fr) 96px 34px;
  align-items: center;
  gap: 12px;
  padding: 9px 12px;
}

.row + .row {
  border-top: 1px solid var(--border);
}

.table-head {
  background: var(--bg-elevated);
  color: var(--text-muted);
  font-size: 10.5px;
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

.label {
  color: var(--text-strong);
  font-size: 13px;
  line-height: 1.35;
  word-break: break-word;
}

.keys {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 5px;
  min-width: 0;
}

.key {
  display: inline-flex;
  align-items: center;
  border: 1px solid var(--border-strong);
  border-radius: 6px;
  background: var(--bg);
  overflow: hidden;
}

.key-label {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  padding: 2px 6px;
  border: none;
  background: transparent;
  color: var(--text);
  cursor: pointer;
}

.key-label:hover {
  background: var(--bg-hover);
}

.key-label:disabled {
  cursor: default;
}

.key-label kbd {
  font-family: var(--mono);
  font-size: 11px;
}

.key-remove {
  display: grid;
  place-items: center;
  height: 100%;
  padding: 0 4px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
}

.key-remove:hover {
  color: var(--danger);
}

.add {
  display: grid;
  place-items: center;
  width: 22px;
  height: 22px;
  border: 1px dashed var(--border-strong);
  border-radius: 6px;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
}

.add:hover {
  color: var(--text-strong);
  border-color: var(--accent);
}

.none {
  color: var(--text-muted);
  font-size: 11px;
  font-style: italic;
}

.scope {
  color: var(--text-muted);
  font-size: 12px;
}

.actions {
  display: flex;
  justify-content: flex-end;
}

.empty {
  margin: 0;
  padding: 14px 12px;
  color: var(--text-muted);
  font-size: 12px;
}

.backdrop {
  position: fixed;
  inset: 0;
  z-index: 60;
  display: grid;
  place-items: center;
  padding: 24px;
  background: rgb(0 0 0 / 55%);
}

.shortcut-modal {
  width: min(430px, 100%);
  padding: 18px;
  border: 1px solid var(--border-strong);
  border-radius: 12px;
  background: var(--bg-panel);
  box-shadow: 0 24px 70px rgb(0 0 0 / 55%);
}

.shortcut-modal:focus {
  outline: none;
}

.shortcut-modal h3 {
  margin: 0 0 14px;
  color: var(--text-strong);
  font-size: 15px;
}

.capture {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  min-height: 72px;
  padding: 12px;
  border: 1px dashed var(--border-strong);
  border-radius: 10px;
  background: var(--bg);
}

.capture kbd {
  padding: 5px 10px;
  border: 1px solid var(--border-strong);
  border-radius: 6px;
  background: var(--bg-elevated);
  color: var(--text-strong);
  font-family: var(--mono);
  font-size: 14px;
}

.capture.waiting {
  border-color: var(--accent);
}

.waiting-text {
  color: var(--text-muted);
  font-size: 12px;
}

.error {
  margin: 8px 0 0;
  color: var(--danger);
  font-size: 12px;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 14px;
}
</style>
