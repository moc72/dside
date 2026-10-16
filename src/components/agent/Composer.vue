<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import Icon from "../Icon.vue";
import ContextRing from "./ContextRing.vue";
import { t } from "../../i18n";
import { settings } from "../../stores/settings";
import { modelChoices, modelLabel } from "../../stores/models";
import { abortTurn, engineReady, running, sendPrompt } from "../../composables/useEngine";

const text = ref("");

/**
 * The field grows with what is typed and stops at ten lines — long enough for a real prompt,
 * short enough that the conversation stays on screen. Beyond that it scrolls.
 */
const MAX_ROWS = 10;
const LINE_HEIGHT = 19.5;
const box = ref<HTMLTextAreaElement | null>(null);

function autoGrow(): void {
  const element = box.value;
  if (!element) return;
  element.style.height = "auto";
  element.style.height = `${Math.min(element.scrollHeight, MAX_ROWS * LINE_HEIGHT)}px`;
}

function reset(): void {
  text.value = "";
  void nextTick(() => {
    if (box.value) box.value.style.height = "auto";
  });
}
const model = ref(settings.model);

const canSend = computed(() => text.value.trim().length > 0 && !running.value);

watch(model, (value) => {
  settings.model = value;
});

function submit() {
  const value = text.value.trim();
  if (!value || running.value) return;
  reset();
  void sendPrompt(value);
}

function toggleAuthorization() {
  settings.fullAuthorization = !settings.fullAuthorization;
}
</script>

<template>
  <div class="composer-wrap">
    <form class="composer" @submit.prevent="submit">
      <textarea
        ref="box"
        v-model="text"
        class="input"
        rows="1"
        :placeholder="t('composer.placeholder')"
        @keydown.enter.exact.prevent="submit"
        @input="autoGrow"
      ></textarea>

      <div class="row">
        <div class="left">
          <button class="chip" type="button" :title="t('composer.attach')">
            <Icon name="plus" :size="14" />
          </button>
          <button
            class="chip"
            :class="{ on: settings.fullAuthorization }"
            type="button"
            @click="toggleAuthorization"
          >
            <Icon name="external-link" :size="14" />
            <span>{{ t("composer.fullAuthorization") }}</span>
            <Icon name="chevron-up" :size="12" />
          </button>
        </div>

        <div class="right">
          <ContextRing />
          <select v-model="model" class="model" :title="`${t('composer.model')}: ${modelLabel(model)} (${model})`">
            <option
              v-if="!modelChoices().some((entry) => entry.id === model)"
              :value="model"
            >
              {{ model }}
            </option>
            <option v-for="entry in modelChoices()" :key="entry.id" :value="entry.id">
              {{ entry.name }}
            </option>
          </select>

          <button
            v-if="running"
            class="send abort"
            type="button"
            :title="t('engine.abort')"
            @click="abortTurn"
          >
            <Icon name="x" :size="15" />
          </button>
          <button
            v-else
            class="send"
            type="submit"
            :disabled="!canSend"
            :title="t('composer.send')"
          >
            <Icon name="send" :size="16" />
          </button>
        </div>
      </div>
    </form>

    <p class="disclaimer">
      {{ engineReady ? t("composer.disclaimer") : t("engine.needsKey") }}
    </p>
  </div>
</template>

<style scoped>
.composer-wrap {
  max-width: 980px;
  margin: 0 auto;
  padding: 0 6px 10px;
}

.composer {
  border: 1px solid var(--border);
  border-radius: 12px;
  background: var(--bg-panel);
  padding: 10px 12px 8px;
}

.input {
  width: 100%;
  min-height: 42px;
  max-height: 195px;
  overflow-y: auto;
  resize: none;
  border: none;
  outline: none;
  background: transparent;
  color: var(--text);
  font-size: 13px;
  line-height: 1.5;
}

.row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-top: 6px;
}

.left,
.right {
  display: flex;
  align-items: center;
  gap: 6px;
}

.chip {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 26px;
  padding: 0 9px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-muted);
  font-size: 12px;
}

.chip:hover {
  background: var(--bg-hover);
  color: var(--text-strong);
}

.chip.on {
  color: var(--accent);
}

.model {
  width: 150px;
  height: 26px;
  padding: 0 6px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text);
  font-size: 12px;
  text-align: right;
}

.model:focus {
  outline: none;
  background: var(--bg-input);
}

.send {
  display: grid;
  place-items: center;
  width: 30px;
  height: 30px;
  border: none;
  border-radius: 50%;
  background: var(--bg-active);
  color: var(--text-strong);
}

.send:disabled {
  opacity: 0.4;
  cursor: default;
}

.send:not(:disabled):hover {
  background: var(--accent-strong);
  color: var(--accent-contrast);
}

.send.abort {
  background: var(--danger);
  color: var(--accent-contrast);
}

.disclaimer {
  margin: 8px 0 0;
  color: var(--text-muted);
  font-size: 11px;
  text-align: center;
}
</style>
