<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import Icon from "../Icon.vue";
import MessageList from "./MessageList.vue";
import Composer from "./Composer.vue";
import { t } from "../../i18n";
import { activeSessionId, messages } from "../../stores/sessions";
import { initEngine } from "../../composables/useEngine";

const scrollEl = ref<HTMLElement | null>(null);
const following = ref(true);
let frame = 0;

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

function queueFollow(): void {
  if (frame) return;
  frame = requestAnimationFrame(() => {
    frame = 0;
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
  void initEngine().then(() => jumpToBottom(false));
});

onBeforeUnmount(() => {
  if (frame) cancelAnimationFrame(frame);
});
</script>

<template>
  <section class="agent">
    <div class="scroll-wrap">
      <div ref="scrollEl" class="scroll" @scroll="onScroll">
        <MessageList :messages="messages" />
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

    <Composer />
  </section>
</template>

<style scoped>
.agent {
  display: flex;
  flex-direction: column;
  background: transparent;
  padding-inline: 16px;
}

.scroll-wrap {
  position: relative;
  flex: 1;
  min-height: 0;
  display: flex;
}

.scroll {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
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
</style>
