<script setup lang="ts">
import { ref } from "vue";
import Icon from "../Icon.vue";
import { t } from "../../i18n";
import { settings } from "../../stores/settings";
import { escapeHtml, renderMarkdown } from "../../lib/preview";
import type { ChatMessage } from "../../types";

defineProps<{
  messages: ChatMessage[];
}>();

/**
 * Tool rows open on demand. The chip alone says "bash ran" — which is exactly what makes
 * someone suspicious. The command and its output belong behind one click, and so does the
 * reason it ran without asking.
 */
const open = ref<Record<string, boolean>>({});

function isOpen(id: string): boolean {
  return open.value[id] === true;
}

function toggle(id: string): void {
  open.value[id] = !open.value[id];
}

/** The user's text stays exactly as typed; only the answer is rendered as markdown. */
function escape(text: string): string {
  return escapeHtml(text).replace(/\n/g, "<br />");
}
</script>

<template>
  <div class="messages">
    <div v-for="message in messages" :key="message.id" class="row" :class="message.role">
      <img v-if="message.role === 'assistant'" class="avatar" src="/whale.png" alt="" />
      <span v-else class="avatar spacer"></span>
      <div class="bubble">
        <!-- The answer as a timeline: what was said and what was run, in that order. -->
        <template v-for="(part, index) in message.parts ?? []" :key="index">
          <details
            v-if="part.kind === 'reasoning'"
            class="reasoning"
            :open="message.streaming && index === (message.parts?.length ?? 0) - 1"
          >
            <summary>{{ t("message.reasoning") }}</summary>
            <div class="reasoning-body">{{ part.text }}</div>
          </details>

          <div
            v-else-if="part.kind === 'text'"
            class="text"
            :class="{ streaming: message.streaming }"
            :title="message.role === 'user' ? undefined : t('message.raw')"
            v-html="
              message.role === 'user' ? escape(part.text) : renderMarkdown(part.text, true)
            "
          ></div>

          <div v-else class="tools">
            <button
              class="tool"
              :class="{ error: part.tool.isError, running: part.tool.running }"
              type="button"
              @click="toggle(part.tool.id)"
            >
              <Icon :name="isOpen(part.tool.id) ? 'chevron-down' : 'chevron-right'" :size="12" />
              <Icon
                :name="part.tool.running ? 'reload' : part.tool.isError ? 'x' : 'check'"
                :size="12"
              />
              <span class="tool-name">{{ part.tool.name }}</span>
              <span class="tool-state">
                {{
                  part.tool.running
                    ? t("engine.running")
                    : part.tool.isError
                      ? t("engine.failed")
                      : t("engine.done")
                }}
              </span>
              <span v-if="part.tool.durationMs" class="tool-time">{{ part.tool.durationMs }}ms</span>
            </button>

            <div v-if="isOpen(part.tool.id)" class="tool-detail">
              <div v-if="part.tool.arguments" class="tool-command">{{ part.tool.arguments }}</div>
              <pre v-if="part.tool.output" class="tool-output">{{ part.tool.output }}</pre>
              <p v-if="settings.fullAuthorization" class="tool-note">
                {{ t("message.autoApproved") }}
              </p>
            </div>
          </div>

        </template>

        <div v-if="message.streaming && !(message.parts ?? []).length" class="pending">
          <span class="dot"></span><span class="dot"></span><span class="dot"></span>
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
  max-width: 980px;
  margin: 0 auto;
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

/* A whole conversation is easier to read in one column, and a question is never as long as
   an answer: the assistant gets the column, the user 75% of it. */
.bubble {
  max-width: 100%;
}

.row.user .bubble {
  max-width: 75%;
}

.text {
  line-height: 1.4;
  padding: 6px 8px;
  white-space: pre-wrap;
  border: 1px solid var(--bubble-border);
  border-radius: var(--radius);
  background-color: var(--bubble-bg);
  font-size: var(--reading-font-size);
}

.row.user .text {
  padding: 6px 8px;
  border: 1px solid var(--bubble-border);
  border-radius: var(--radius);
  background: var(--bubble-user-bg);
}

/* Empty seat for the whale: user messages line up with the partner's. */
.avatar.spacer {
  background: none;
}

.pending {
  display: flex;
  gap: 4px;
  padding: 6px 0;
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
  font-size: max(11px, calc(var(--reading-font-size) - 2px));
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

.tool-detail {
  margin: 6px 0 0 18px;
  padding: 8px 10px;
  border: 1px solid var(--border);
  border-left: 2px solid var(--accent);
  border-radius: var(--radius-sm);
  background: var(--bg);
}

.tool-command {
  color: var(--text);
  font-family: var(--mono);
  font-size: 11.5px;
  word-break: break-word;
}

.tool-output {
  margin: 6px 0 0;
  max-height: 260px;
  overflow: auto;
  color: var(--text-muted);
  font-family: var(--mono);
  font-size: 11.5px;
  line-height: 1.45;
  white-space: pre-wrap;
  word-break: break-word;
}

.tool-note {
  margin: 8px 0 0;
  color: var(--text-muted);
  font-size: 11px;
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
  font-family: inherit;
  font-size: 12px;
  width: fit-content;
  cursor: pointer;
}

.tool:hover {
  border-color: var(--border-strong);
  color: var(--text);
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
