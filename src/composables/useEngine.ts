import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { EngineEvent, Session, SessionGroup } from "../types";
import {
  activeSessionId,
  appendAssistantDelta,
  appendAssistantReasoning,
  appendUserMessage,
  beginAssistantMessage,
  clearConversation,
  endAssistantMessage,
  failAssistantMessage,
  messagesBySession,
  selectSession,
  sessions,
  upsertToolActivity,
} from "../stores/sessions";
import { settings } from "../stores/settings";
import { ensureWorkspace } from "./useWorkspace";
import { buildPersona } from "../engine/persona";
import { t } from "../i18n";

interface DbSessionRow {
  id: string;
  title: string;
  group: string;
  workspace: string;
  updatedAt: number;
}

interface DbToolRow {
  id: string;
  name: string;
  arguments: string | null;
  output: string | null;
  isError: boolean;
  durationMs: number | null;
}

interface DbHistoryRow {
  id: string;
  role: string;
  text: string;
  tools: DbToolRow[];
}

export const running = ref(false);
export const engineReady = ref(false);
export const lastError = ref<string | null>(null);
export const secretBackend = ref<"keyring" | "file" | "none">("none");

/** Bumped whenever the partner rewrites its plan, so the Progress panel can reload. */
export const todosRevision = ref(0);

/** Set while a turn is blocked on the user (`ask_user`). Cleared by answering. */
export const pendingAction = ref<EngineEvent["action"] | null>(null);

type SecretStatus = { present: boolean; backend: string };

/** The API key used to live in localStorage. Move it to the keyring once, then forget it. */
const LEGACY_SETTINGS_KEY = "ds-ide.settings";
let legacyKey: string | null = null;

function extractLegacyKey(): void {
  try {
    const raw = localStorage.getItem(LEGACY_SETTINGS_KEY);
    if (!raw) return;
    const parsed = JSON.parse(raw) as Record<string, unknown>;
    const value = parsed.apiKey;
    if (typeof value !== "string" || !value.trim()) return;
    legacyKey = value;
    delete parsed.apiKey;
    localStorage.setItem(LEGACY_SETTINGS_KEY, JSON.stringify(parsed));
  } catch {
    // unreadable legacy blob — nothing to migrate
  }
}

function applySecret(status: SecretStatus): void {
  engineReady.value = status.present;
  secretBackend.value = (status.backend as "keyring" | "file" | "none") ?? "none";
}

export async function refreshSecret(): Promise<void> {
  const status = await invoke<SecretStatus>("secret_status");
  if (!status.present && legacyKey) {
    const migrated = await invoke<SecretStatus>("secret_set", { value: legacyKey });
    legacyKey = null;
    applySecret(migrated);
    return;
  }
  applySecret(status);
}

export async function saveApiKey(value: string): Promise<void> {
  applySecret(await invoke<SecretStatus>("secret_set", { value }));
}

export async function clearApiKey(): Promise<void> {
  applySecret(await invoke<SecretStatus>("secret_clear"));
}

let unlisten: (() => void) | null = null;

export async function configureEngine(): Promise<void> {
  await invoke("engine_set_config", {
    config: {
      baseUrl: settings.baseUrl || "https://api.deepseek.com",
      model: settings.model || "deepseek-chat",
      systemPrompt: buildPersona("Seeky", settings.workspace),
      workspace: settings.workspace,
      fullAuthorization: settings.fullAuthorization,
      maxTurns: 24,
    },
  });
  await refreshSecret();
}

function handleEvent(event: EngineEvent): void {
  if (event.sessionId !== activeSessionId.value) return;

  switch (event.kind) {
    case "stream":
      if (event.text) appendAssistantDelta(event.text);
      break;
    case "reasoning":
      if (event.text) appendAssistantReasoning(event.text);
      break;
    case "tool_start":
      if (event.tool) {
        upsertToolActivity({
          id: event.tool.id,
          name: event.tool.name,
          arguments: event.tool.arguments,
          running: true,
        });
      }
      break;
    case "tool_result":
      if (event.tool) {
        upsertToolActivity({
          id: event.tool.id,
          name: event.tool.name,
          output: event.tool.output,
          isError: event.tool.isError,
          durationMs: event.tool.durationMs,
          running: false,
        });
        if (event.tool.name === "todowrite") todosRevision.value += 1;
      }
      break;
    case "error":
      lastError.value = event.error ?? "unknown engine error";
      failAssistantMessage(lastError.value);
      pendingAction.value = null;
      running.value = false;
      break;
    case "action_required":
      pendingAction.value = event.action ?? null;
      break;
    case "title": {
      const session = sessions.value.find((entry) => entry.id === event.sessionId);
      if (session && event.text) session.title = event.text;
      break;
    }
    case "turn_end":
      endAssistantMessage(event.stopReason);
      pendingAction.value = null;
      running.value = false;
      break;
    default:
      break;
  }
}

export async function initEngine(): Promise<void> {
  if (!unlisten) {
    unlisten = await listen<EngineEvent>("engine://event", (event) => handleEvent(event.payload));
  }
  extractLegacyKey();
  await ensureWorkspace();
  await configureEngine();
  await loadSessions();
  await loadHistory(activeSessionId.value);
}

export async function loadSessions(): Promise<void> {
  const rows = await invoke<DbSessionRow[]>("sessions_list");

  if (rows.length === 0) {
    const id = `s${Date.now()}`;
    await invoke("session_create", { id, title: "", workspace: settings.workspace });
    sessions.value = [
      { id, title: "Neue Aufgabe", group: "project", workspace: settings.workspace },
    ];
    activeSessionId.value = id;
    return;
  }

  sessions.value = rows.map(
    (row): Session => ({
      id: row.id,
      title: row.title || "Neue Aufgabe",
      group: row.group as SessionGroup,
      workspace: row.workspace,
    }),
  );

  if (!rows.some((row) => row.id === activeSessionId.value)) {
    activeSessionId.value = rows[0].id;
  }
}

export async function loadHistory(sessionId: string): Promise<void> {
  const rows = await invoke<DbHistoryRow[]>("session_history", { sessionId });
  messagesBySession[sessionId] = rows.map((row) => ({
    id: row.id,
    role: row.role === "user" ? "user" : "assistant",
    text: row.text,
    tools: row.tools.map((tool) => ({
      id: tool.id,
      name: tool.name,
      arguments: tool.arguments ?? undefined,
      output: tool.output ?? undefined,
      isError: tool.isError,
      durationMs: tool.durationMs ?? undefined,
    })),
  }));
}

export async function openSession(sessionId: string): Promise<void> {
  selectSession(sessionId);
  await loadHistory(sessionId);
}

export async function createSession(): Promise<void> {
  const id = `s${Date.now()}`;
  await invoke("session_create", { id, title: "", workspace: settings.workspace });
  await loadSessions();
  activeSessionId.value = id;
  clearConversation();
}

export async function sendPrompt(text: string): Promise<void> {
  if (running.value) return;
  lastError.value = null;

  await configureEngine();
  if (!engineReady.value) {
    lastError.value = t("engine.needsKey");
    return;
  }

  appendUserMessage(text);
  beginAssistantMessage();
  running.value = true;

  try {
    await invoke("engine_send", { sessionId: activeSessionId.value, prompt: text });
  } catch (error) {
    running.value = false;
    lastError.value = String(error);
    failAssistantMessage(lastError.value);
  }
}

export async function abortTurn(): Promise<void> {
  await invoke("engine_abort", { sessionId: activeSessionId.value });
}

/** Answer a pending question so the blocked turn can continue. */
export async function answerAction(actionId: string, answer: string): Promise<void> {
  pendingAction.value = null;
  await invoke("engine_answer", { actionId, answer });
}

export async function resetConversation(): Promise<void> {
  await invoke("engine_reset", { sessionId: activeSessionId.value });
}
