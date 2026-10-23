<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import Icon from "../Icon.vue";
import MessageList from "./MessageList.vue";
import Composer from "./Composer.vue";
import { t } from "../../i18n";
import { activeSessionId, messages } from "../../stores/sessions";
import { answerAction, initEngine, pendingAction, running } from "../../composables/useEngine";

const composer = ref<InstanceType<typeof Composer> | null>(null);

/**
 * After a turn the next sentence usually goes into the same field. Put the cursor back —
 * but never take it away from a field the user is already typing in (for example while
 * the turn is still streaming and they are writing the next message).
 */
watch(running, (isRunning) => {
  if (isRunning) return;
  const active = document.activeElement as HTMLElement | null;
  if (
    active &&
    (active.tagName === "INPUT" ||
      active.tagName === "TEXTAREA" ||
      active.tagName === "SELECT" ||
      active.isContentEditable)
  ) {
    return;
  }
  void nextTick(() => composer.value?.focus());
});

const freeAnswer = ref("");

async function answer(option: string) {
  const action = pendingAction.value;
  if (!action) return;
  await answerAction(action.id, option);
}

async function submitFree() {
  const text = freeAnswer.value.trim();
  if (!text) return;
  freeAnswer.value = "";
  await answer(text);
}

const scrollEl = ref<HTMLElement | null>(null);
const listEl = ref<InstanceType<typeof MessageList> | null>(null);
const following = ref(true);
let observer: ResizeObserver | null = null;

const messagesChanged = computed(() => {
  const list = messages.value;
  const last = list[list.length - 1];
  const tools = last?.tools ?? [];
  const toolBytes = tools.reduce((sum, tool) => sum + (tool.output?.length ?? 0), 0);
  return `${list.length}|${last?.text.length ?? 0}|${tools.length}|${toolBytes}`;
});

function isAtBottom(element: HTMLElement): boolean {
  return element.scrollHeight - element.scrollTop - element.clientHeight < 48;
}

/**
 * Scrolls to the newest line. Deliberately not `requestAnimationFrame`: in this webview (splash
 * included) rAF can go quiet, which is why the list stopped following until it was nudged.
 * `nextTick` runs after Vue has written the DOM, which is exactly the moment we need.
 */
function queueFollow(): void {
  void nextTick(() => {
    const element = scrollEl.value;
    if (element) element.scrollTop = element.scrollHeight;
  });
}

function onScroll(): void {
  const element = scrollEl.value;
  if (!element) return;
  following.value = isAtBottom(element);
}

function jumpToBottom(smooth: boolean): void {
  const element = scrollEl.value;
  if (!element) return;
  following.value = true;
  if (smooth) {
    element.scrollTo({ top: element.scrollHeight, behavior: "smooth" });
    return;
  }
  element.scrollTop = element.scrollHeight;
}

watch(
  messagesChanged,
  () => {
    const list = messages.value;
    // The user just acted: always follow their own message.
    if (list[list.length - 1]?.role === "user") following.value = true;
    if (following.value) queueFollow();
  },
  { flush: "post" },
);

watch(activeSessionId, () => {
  following.value = true;
  queueFollow();
});

onMounted(() => {
  // Markdown, images and fonts change the height after the message was appended. As long as
  // the reader is at the bottom, follow that growth too.
  const target = (listEl.value?.$el as HTMLElement | undefined) ?? scrollEl.value;
  if (target && typeof ResizeObserver !== "undefined") {
    observer = new ResizeObserver(() => {
      if (following.value) queueFollow();
    });
    observer.observe(target);
  }
  void initEngine().then(() => jumpToBottom(false));
});

onBeforeUnmount(() => {
  observer?.disconnect();
  observer = null;
});
</script>

<template>
  <section class="agent">
    <div class="scroll-wrap">
      <div ref="scrollEl" class="scroll" @scroll="onScroll">
        <MessageList ref="listEl" :messages="messages" />
      </div>

      <button
        v-if="!following"
        class="jump"
        type="button"
        @click="jumpToBottom(true)"
      >
        <Icon name="chevron-down" :size="13" />
        <span>{{ t("composer.scrollToBottom") }}</span>
      </button>
    </div>

    <section v-if="pendingAction" class="question">
      <div class="question-head">
        <Icon :name="pendingAction.kind === 'permission' ? 'rudder' : 'sparkle'" :size="13" />
        <span>{{
          pendingAction.kind === "permission"
            ? t("permission.requestTitle")
            : t("question.title")
        }}</span>
      </div>
      <p
        class="question-text"
        :class="{ technical: pendingAction.kind === 'permission' }"
      >{{ pendingAction.question }}</p>

      <template v-if="pendingAction.kind === 'permission'">
        <p class="question-note">{{ t("permission.onceHint") }}</p>
        <div class="options">
          <button type="button" @click="answer('allow')">
            {{ t("permission.allow") }}
          </button>
          <button
            v-if="pendingAction.options.includes('session')"
            type="button"
            @click="answer('session')"
          >
            {{ t("permission.allowSession") }}
          </button>
          <button type="button" class="deny" @click="answer('deny')">
            {{ t("permission.deny") }}
          </button>
        </div>
      </template>

      <template v-else>
        <div v-if="pendingAction.options.length" class="options">
          <button
            v-for="option in pendingAction.options"
            :key="option"
            type="button"
            @click="answer(option)"
          >
            {{ option }}
          </button>
        </div>
        <form class="free" @submit.prevent="submitFree">
          <input v-model="freeAnswer" :placeholder="t('question.placeholder')" />
          <button type="submit" :disabled="!freeAnswer.trim()">{{ t("question.send") }}</button>
        </form>
      </template>
    </section>

    <Composer ref="composer" />
  </section>
</template>

<style scoped>
.agent {
  display: flex;
  flex-direction: column;
  padding-inline: 16px;
  position: relative;
  /* One reading column for the list and the composer. It keeps the 70 % feel on a wide
     window but refuses to shrink below a usable width when the side panels are open —
     which is what clipped the composer controls at the minimum window size. */
  --chat-width: min(100%, max(70%, 560px), 900px);
  /* Own stacking context, so the watermark can sit *behind* the content instead of above it:
     a positioned ::before with z-index auto paints over in-flow children — which is why the
     mark showed on top of the supposedly opaque composer. */
  isolation: isolate;
}

/* Watermark behind the conversation. The DeepSeek mark, white on dark and blue on light. */
.agent::before {
  content: "";
  position: absolute;
  inset: 0;
  z-index: -1;
  background: var(--watermark) center 30% / var(--watermark-size) auto no-repeat;
  opacity: var(--watermark-opacity);
  pointer-events: none;
}

.scroll-wrap {
  position: relative;
  flex: 1;
  min-height: 0;
  display: flex;
  justify-content: center;
}

/* The scrolling column *is* the column the composer uses, so both have the same edges.
   Before, the scrollbar ate into the messages and made them a scrollbar narrower. */
.scroll {
  /* No `flex: 1` here: as a flex item its basis would win over the width, and the column
     would quietly stretch to the whole panel. */
  min-height: 0;
  width: var(--chat-width);
  margin: 0 auto;
  padding: 0 6px;
  overflow-y: auto;
  scrollbar-width: thin;
  scrollbar-color: var(--border-strong) transparent;
  scrollbar-gutter: stable;
}

.scroll::-webkit-scrollbar {
  width: 6px;
}

.scroll::-webkit-scrollbar-thumb {
  background: var(--border-strong);
  border-radius: 3px;
}

.scroll::-webkit-scrollbar-track {
  background: transparent;
}

.jump {
  position: absolute;
  left: 50%;
  bottom: 12px;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 12px;
  border: 1px solid var(--border-strong);
  border-radius: 999px;
  background: var(--bg-elevated);
  color: var(--text-strong);
  font-size: 12px;
  box-shadow: 0 8px 20px rgba(0, 0, 0, 0.45);
}

.jump:hover {
  border-color: var(--accent);
}

.question {
  margin: 0 14px 12px;
  padding: 12px 14px;
  border: 1px solid var(--accent-strong);
  border-radius: 12px;
  background: var(--bg-elevated);
  font-size: var(--reading-font-size);
}

.question-head {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--accent);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.5px;
  text-transform: uppercase;
}

.question-text {
  margin: 8px 0 10px;
  color: var(--text-strong);
  font-size: 13px;
  line-height: 1.5;
}

.question-text.technical {
  max-height: 180px;
  overflow: auto;
  font-family: var(--mono);
  font-size: 11px;
  line-height: 1.45;
  white-space: pre-wrap;
  word-break: break-word;
}

.question-note {
  margin: 0 0 8px;
  color: var(--text-muted);
  font-size: 11px;
}

.options {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 10px;
}

.options button {
  padding: 5px 10px;
  border: 1px solid var(--border-strong);
  border-radius: 999px;
  background: transparent;
  color: var(--text);
  font-size: 12px;
}

.options button:hover {
  border-color: var(--accent);
  color: var(--text-strong);
}

.options button.deny:hover {
  border-color: var(--danger);
  color: var(--danger);
}

.free {
  display: flex;
  gap: 6px;
}

.free input {
  flex: 1;
  min-width: 0;
  height: 28px;
  padding: 0 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-input);
  color: var(--text);
  font-size: 12px;
}

.free button {
  height: 28px;
  padding: 0 12px;
  border: none;
  border-radius: var(--radius-sm);
  background: var(--accent-strong);
  color: var(--accent-contrast);
  font-size: 12px;
}

.free button:disabled {
  opacity: 0.45;
}
</style>
