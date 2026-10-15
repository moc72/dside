<script setup lang="ts">
import Icon from "../Icon.vue";
import { t } from "../../i18n";
import type { ChatMessage } from "../../types";

defineProps<{
  messages: ChatMessage[];
}>();
</script>

<template>
  <div class="messages">
    <div v-for="message in messages" :key="message.id" class="row" :class="message.role">
      <img v-if="message.role === 'assistant'" class="avatar" src="/whale.png" alt="" />
      <div class="bubble">
        <details v-if="message.reasoning" class="reasoning" :open="message.streaming && !message.text">
          <summary>{{ t("message.reasoning") }}</summary>
          <div class="reasoning-body">{{ message.reasoning }}</div>
        </details>

        <div v-if="message.text" class="text" :class="{ streaming: message.streaming }">
          {{ message.text }}
        </div>
        <div v-else-if="message.streaming && !message.reasoning" class="pending">
          <img class="thinking" src="/whale-anim.webp" alt="" />
        </div>

        <div v-if="message.tools && message.tools.length" class="tools">
          <div
            v-for="tool in message.tools"
            :key="tool.id"
            class="tool"
            :class="{ error: tool.isError, running: tool.running }"
            :title="tool.output ?? tool.arguments ?? ''"
          >
            <Icon
              :name="tool.running ? 'reload' : tool.isError ? 'x' : 'check'"
              :size="12"
            />
            <span class="tool-name">{{ tool.name }}</span>
            <span class="tool-state">
              {{
                tool.running
                  ? t("engine.running")
                  : tool.isError
                    ? t("engine.failed")
                    : t("engine.done")
              }}
            </span>
            <span v-if="tool.durationMs" class="tool-time">{{ tool.durationMs }}ms</span>
          </div>
        </div>

        <div v-if="message.error" class="error">{{ message.error }}</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.messages {
  display: flex;
  flex-direction: column;
  gap: 18px;
  padding: 20px 6px;
}

.row {
  display: flex;
  align-items: flex-start;
  gap: 10px;
}

.avatar {
  width: 22px;
  height: 22px;
  flex: none;
  margin-top: 2px;
  object-fit: contain;
}

.row.user {
  justify-content: flex-end;
}

.bubble {
  max-width: 82%;
}

.text {
  line-height: 1.6;
  white-space: pre-wrap;
}

.row.user .text {
  padding: 12px 16px;
  border-radius: var(--radius);
  background: var(--bg-elevated);
}

.pending {
  display: flex;
  gap: 4px;
  padding: 6px 0;
}

/* The partner thinking: the whale swims instead of three blinking dots. */
.thinking {
  width: 34px;
  height: auto;
  object-fit: contain;
}

.row.user .avatar {
  display: none;
}

.reasoning {
  margin-bottom: 10px;
  padding: 8px 12px;
  border-left: 2px solid var(--border-strong);
  border-radius: var(--radius-sm);
  background: var(--bg-panel);
}

.reasoning summary {
  cursor: pointer;
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.5px;
  text-transform: uppercase;
  user-select: none;
}

.reasoning-body {
  margin-top: 8px;
  color: var(--text-muted);
  font-size: 12px;
  font-style: italic;
  line-height: 1.55;
  white-space: pre-wrap;
  max-height: 340px;
  overflow-y: auto;
}

.dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--text-muted);
  animation: pulse 1.2s infinite ease-in-out;
}

.dot:nth-child(2) {
  animation-delay: 0.15s;
}

.dot:nth-child(3) {
  animation-delay: 0.3s;
}

@keyframes pulse {
  0%,
  100% {
    opacity: 0.25;
  }
  50% {
    opacity: 1;
  }
}

.tools {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-top: 10px;
}

.tool {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 9px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-panel);
  color: var(--text-muted);
  font-size: 12px;
  width: fit-content;
}

.tool.running {
  color: var(--accent);
}

.tool.error {
  color: var(--danger);
  border-color: var(--danger);
}

.tool-name {
  color: var(--text);
  font-family: var(--mono);
}

.tool-time {
  opacity: 0.6;
}

.error {
  margin-top: 10px;
  padding: 8px 10px;
  border-left: 2px solid var(--danger);
  background: var(--bg-panel);
  color: var(--danger);
  font-size: 12px;
  white-space: pre-wrap;
}
</style>
