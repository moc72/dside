<script setup lang="ts">
import { computed, ref } from "vue";
import Icon from "../Icon.vue";
import MarkdownText from "./MarkdownText.vue";
import { t } from "../../i18n";
import { settings } from "../../stores/settings";
import { activeSessionId } from "../../stores/sessions";
import { openFileHistory } from "../../stores/workArea";
import { resolveWorkspacePath } from "../../composables/useFollow";
import { resumesBySession, retryTurn } from "../../composables/useEngine";
import type { ChatMessage } from "../../types";

defineProps<{
  messages: ChatMessage[];
}>();

/** The compressed summary of the open session, if it has been compressed. */
const resume = computed(() => resumesBySession[activeSessionId.value] ?? null);

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

/**
 * Reasoning stays collapsed while it is being produced. Watching it grow is what made the
 * whole panel stutter: every chunk re-laid out a block that was already scrolled anyway.
 * Once the turn is over it can be opened like a normal disclosure.
 */
function keepClosed(event: Event, streaming?: boolean): void {
  const element = event.target as HTMLDetailsElement;
  if (streaming && element.open) element.open = false;
}

/** Opens the file tab with the snapshot history for a write/edit tool call. */
function showChange(tool: { name: string; arguments?: string }): void {
  if (!tool.arguments) return;
  try {
    const args = JSON.parse(tool.arguments) as { path?: unknown };
    if (typeof args.path !== "string" || !args.path.trim()) return;
    openFileHistory(resolveWorkspacePath(args.path));
  } catch {
    // The model can emit invalid JSON; there is nothing to open then.
  }
}

function retryMessage(id: string): void {
  void retryTurn(id);
}
</script>

<template>
  <div class="messages">
    <details v-if="resume" class="resume">
      <summary>
        <Icon name="clock" :size="13" />
        <span class="resume-title">{{ t("resume.title") }}</span>
        <span class="resume-hint">{{ t("resume.hint") }}</span>
      </summary>
      <div class="resume-body">{{ resume.summary }}</div>
    </details>

    <div v-for="message in messages" :key="message.id" class="row" :class="message.role">
      <!-- <img v-if="message.role === 'assistant'" class="avatar" src="/whale.png" alt="" />
      <span v-else class="avatar spacer"></span> -->
      <div class="bubble">
        <!-- The answer as a timeline: what was said and what was run, in that order. -->
        <template v-for="(part, index) in message.parts ?? []" :key="index">
          <details
            v-if="part.kind === 'reasoning'"
            class="reasoning"
            :class="{ live: message.streaming }"
            @toggle="keepClosed($event, message.streaming)"
          >
            <summary>
              <span>{{ t("message.reasoning") }}</span>
              <span class="reasoning-count">
                {{ part.text.length.toLocaleString() }} {{ t("message.characters") }}
              </span>
            </summary>
            <div class="reasoning-body">{{ part.text }}</div>
          </details>

          <img
            v-else-if="part.kind === 'image' && part.preview"
            class="attachment"
            :src="part.preview"
            :alt="part.name"
            :title="part.name"
          />

          <!-- A picture without a stored preview still says which file it was. -->
          <div v-else-if="part.kind === 'image'" class="attachment-name">{{ part.name }}</div>

          <MarkdownText
            v-else-if="part.kind === 'text'"
            :text="part.text"
            :user="message.role === 'user'"
            :streaming="message.streaming"
            :tooltip="message.role === 'user' ? undefined : t('message.raw')"
          />

          <!-- No wrapper per tool: one margin on the chip itself, so two chips in a row are
               exactly one gap apart instead of one per wrapper plus one per element. -->
          <template v-else>
            <button
              class="tool"
              :class="{
                error: part.tool.isError,
                running: part.tool.running,
                cancelled: part.tool.cancelled,
              }"
              type="button"
              @click="toggle(part.tool.id)"
            >
              <Icon :name="isOpen(part.tool.id) ? 'chevron-down' : 'chevron-right'" :size="12" />
              <Icon
                :name="
                  part.tool.running
                    ? 'reload'
                    : part.tool.cancelled || part.tool.isError
                      ? 'x'
                      : 'check'
                "
                :size="12"
              />
              <span class="tool-name">{{ part.tool.name }}</span>
              <span class="tool-state">
                {{
                  part.tool.running
                    ? t("engine.running")
                    : part.tool.cancelled
                      ? t("engine.cancelled")
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
              <img
                v-if="part.tool.image"
                class="tool-image"
                :src="part.tool.image"
                alt="Screenshot"
              />
              <button
                v-if="
                  (part.tool.name === 'write' || part.tool.name === 'edit') &&
                  !part.tool.isError &&
                  part.tool.arguments
                "
                class="tool-diff"
                type="button"
                @click="showChange(part.tool)"
              >
                <Icon name="clock" :size="13" />
                <span>{{ t("viewFile.history") }}</span>
              </button>
              <p v-if="settings.permission === 'full'" class="tool-note">
                {{ t("message.autoApproved") }}
              </p>
            </div>
          </template>

        </template>

        <div v-if="message.streaming && !(message.parts ?? []).length" class="pending">
          <span class="dot"></span><span class="dot"></span><span class="dot"></span>
        </div>

        <div v-if="message.error" class="error">
          <span class="error-text">{{ message.error }}</span>
          <button class="retry" type="button" @click="retryMessage(message.id)">
            {{ t("engine.retry") }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.messages {
  display: flex;
  flex-direction: column;
  padding: 20px 6px;
}

/* The compressed past of the conversation, one click away. It sits above the live timeline
   rather than replacing it: the recent turns are still the context that matters. */
.resume {
  position: sticky;
  top: 0;
  z-index: 5;
  margin: 0 0 14px;
  padding: 10px 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg-panel);
  color: var(--text);
}

.resume summary {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  color: var(--text-strong);
  font-size: 12px;
}

.resume-title {
  font-weight: 600;
}

.resume-hint {
  color: var(--text-muted);
  font-size: 11px;
}

.resume-body {
  margin-top: 8px;
  max-height: 220px;
  overflow-y: auto;
  color: var(--text);
  font-size: 12px;
  line-height: 1.55;
  white-space: pre-wrap;
}

.row {
  display: flex;
  align-items: flex-start;
  min-width: 0;
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
  width: 100%;
  min-width: 0;
  margin-top: 10px;
}

.row.user .bubble {
  max-width: 70%;
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

.reasoning.live summary {
  cursor: default;
}

.reasoning-count {
  margin-left: 8px;
  font-weight: 500;
  letter-spacing: 0;
  text-transform: none;
  opacity: 0.75;
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

.attachment-name {
  margin: 0 0 8px;
  color: var(--text-muted);
  font-size: 11.5px;
}

.attachment {
  display: block;
  max-width: min(420px, 100%);
  max-height: 320px;
  margin: 0 0 8px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius);
  object-fit: contain;
}

.tool-detail {
  margin-top: 6px;
  padding: 8px 10px;
  border: 1px solid var(--border);
  border-left: 2px solid var(--accent);
  border-radius: var(--radius-sm);
  background: var(--bg);
  align-items: end;
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

.tool-image {
  display: block;
  max-width: 100%;
  margin-top: 8px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
}

.tool-note {
  margin: 8px 0 0;
  color: var(--text-muted);
  font-size: 11px;
}

.tool-diff {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  margin-top: 8px;
  padding: 3px 8px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text);
  font-size: 11px;
}

.tool-diff:hover {
  background: var(--bg-hover);
}

.tool {
  display: flex;
  align-items: center;
  gap: 8px;
  /* Full width, so the row reads as a row: icons left, what it did at the right end. */
  width: 100%;
  margin-top: 10px;
  padding: 5px 9px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-panel);
  color: var(--text-muted);
  font-family: inherit;
  font-size: 12px;
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

.tool.cancelled {
  color: var(--text-muted);
}

/* Name, state and duration sit at the right end of the row; the two icons stay left. */
.tool-name {
  margin-left: auto;
  color: var(--text);
  font-family: var(--mono);
}

.tool-time {
  opacity: 0.6;
}

.error {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  margin-top: 10px;
  padding: 8px 10px;
  border-left: 2px solid var(--danger);
  background: var(--bg-panel);
  color: var(--danger);
  font-size: 12px;
  white-space: pre-wrap;
}

.error-text {
  flex: 1;
  min-width: 0;
}

.retry {
  flex: none;
  height: 24px;
  padding: 0 9px;
  border: 1px solid var(--danger);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--danger);
  font-size: 11px;
  white-space: nowrap;
}

.retry:hover {
  background: var(--bg-hover);
}
</style>
