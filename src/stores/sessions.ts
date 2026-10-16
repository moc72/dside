import { computed, reactive, ref, watch } from "vue";
import { saveUiState } from "../lib/uiState";
import type { ChatMessage, Session, ToolActivity } from "../types";

export const sessions = ref<Session[]>([]);

export const activeSessionId = ref<string>("s1");

export const activeSession = computed(
  () => sessions.value.find((session) => session.id === activeSessionId.value) ?? null,
);

const messagesBySession = reactive<Record<string, ChatMessage[]>>({ s1: [] });
export { messagesBySession };

export const messages = computed<ChatMessage[]>(
  () => messagesBySession[activeSessionId.value] ?? [],
);

function bucket(): ChatMessage[] {
  const id = activeSessionId.value;
  return messagesBySession[id] ?? (messagesBySession[id] = []);
}

let seq = 0;
let currentAssistantId: string | null = null;

function currentAssistant(): ChatMessage | undefined {
  if (!currentAssistantId) return undefined;
  return bucket().find((message) => message.id === currentAssistantId);
}

export function appendUserMessage(text: string): void {
  seq += 1;
  bucket().push({ id: `u${seq}`, role: "user", text });
}

export function beginAssistantMessage(): void {
  seq += 1;
  const message: ChatMessage = {
    id: `a${seq}`,
    role: "assistant",
    text: "",
    streaming: true,
    tools: [],
  };
  bucket().push(message);
  currentAssistantId = message.id;
}

export function appendAssistantDelta(chunk: string): void {
  const message = currentAssistant();
  if (message) message.text += chunk;
}

export function appendAssistantReasoning(chunk: string): void {
  const message = currentAssistant();
  if (message) message.reasoning = (message.reasoning ?? "") + chunk;
}

export function upsertToolActivity(activity: ToolActivity): void {
  const message = currentAssistant();
  if (!message) return;
  const tools = message.tools ?? (message.tools = []);
  const index = tools.findIndex((entry) => entry.id === activity.id);
  if (index >= 0) tools[index] = { ...tools[index], ...activity };
  else tools.push(activity);
}

export function endAssistantMessage(stopReason?: string): void {
  const message = currentAssistant();
  if (message) {
    message.streaming = false;
    if (stopReason && stopReason !== "end_turn" && stopReason !== "abort") {
      message.error = stopReason;
    }
  }
  currentAssistantId = null;
}

export function failAssistantMessage(error: string): void {
  const message = currentAssistant();
  if (!message) return;
  message.streaming = false;
  message.error = error;
  currentAssistantId = null;
}

export function clearConversation(): void {
  messagesBySession[activeSessionId.value] = [];
  currentAssistantId = null;
}

export function selectSession(id: string): void {
  activeSessionId.value = id;
  currentAssistantId = null;
}

/**
 * Remembers which conversation was open. Called before the first session list is loaded, so
 * a stale id (deleted, archived) is simply replaced by the newest session at that point.
 */
export function initSessionSelection(stored: unknown): void {
  if (typeof stored === "string" && stored) activeSessionId.value = stored;
  watch(activeSessionId, (id) => saveUiState("activeSession", id));
}
