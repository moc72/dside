import { computed, reactive, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { loadJson } from "../lib/persist";
import type { AppMode, Appearance } from "../types";

export interface SettingsState {
  mode: AppMode;
  appearance: Appearance;
  showInMenuBar: boolean;
  launchOnStartup: boolean;
  desktopNotifications: boolean;
  joinEarlyAccess: boolean;
  fullAuthorization: boolean;
  pinnedEditorId: string | null;
  workspace: string;
  baseUrl: string;
  model: string;
  displayName: string;
}

/** The API key is deliberately NOT here: it lives in the OS keyring (see secret.rs). */
const PERSISTED_KEYS: (keyof SettingsState)[] = [
  "mode",
  "appearance",
  "showInMenuBar",
  "launchOnStartup",
  "desktopNotifications",
  "joinEarlyAccess",
  "fullAuthorization",
  "pinnedEditorId",
  "workspace",
  "baseUrl",
  "model",
  "displayName",
];

const LEGACY_KEY = "ds-ide.settings";

const defaults: SettingsState = {
  mode: "everyday",
  appearance: "dark",
  showInMenuBar: true,
  launchOnStartup: true,
  desktopNotifications: false,
  joinEarlyAccess: false,
  fullAuthorization: true,
  pinnedEditorId: null,
  workspace: "",
  baseUrl: "https://api.deepseek.com",
  model: "deepseek-chat",
  displayName: "",
};

export const settings = reactive<SettingsState>({ ...defaults });

let loaded = false;
let timer: number | undefined;

async function persistNow(): Promise<void> {
  for (const key of PERSISTED_KEYS) {
    try {
      await invoke("settings_set", { key, value: JSON.stringify(settings[key]) });
    } catch {
      // persistence is best effort; the app must keep working without it
    }
  }
}

function schedulePersist(): void {
  if (!loaded) return;
  if (timer) window.clearTimeout(timer);
  timer = window.setTimeout(() => {
    timer = undefined;
    void persistNow();
  }, 400);
}

function apply(stored: Record<string, unknown>): void {
  const target = settings as unknown as Record<string, unknown>;
  for (const key of PERSISTED_KEYS) {
    if (!(key in stored)) continue;
    target[key] = stored[key];
  }
}

/**
 * Settings live in the SQLite `settings` table. Anything still sitting in localStorage
 * from earlier builds is imported once and then removed.
 */
export async function initSettings(): Promise<void> {
  let stored: Record<string, string> = {};
  try {
    stored = await invoke<Record<string, string>>("settings_all");
  } catch {
    stored = {};
  }

  if (Object.keys(stored).length === 0) {
    const legacy = loadJson<Record<string, unknown> | null>(LEGACY_KEY, null);
    if (legacy && typeof legacy === "object") {
      apply(legacy);
      try {
        localStorage.removeItem(LEGACY_KEY);
      } catch {
        // storage unavailable — the import still happened
      }
      loaded = true;
      watch(settings, schedulePersist, { deep: true });
      await persistNow();
      return;
    }
  }

  const decoded: Record<string, unknown> = {};
  for (const [key, raw] of Object.entries(stored)) {
    try {
      decoded[key] = JSON.parse(raw);
    } catch {
      // skip unreadable values rather than breaking startup
    }
  }
  apply(decoded);

  loaded = true;
  watch(settings, schedulePersist, { deep: true });
}

const media =
  typeof window === "undefined" ? null : window.matchMedia("(prefers-color-scheme: dark)");

function prefersDark(): boolean {
  return media?.matches ?? true;
}

export const resolvedTheme = computed<"light" | "dark">(() =>
  settings.appearance === "system" ? (prefersDark() ? "dark" : "light") : settings.appearance,
);

function applyTheme(): void {
  if (typeof document !== "undefined") {
    document.documentElement.dataset.theme = resolvedTheme.value;
  }
}

export function initTheme(): void {
  applyTheme();
  watch(resolvedTheme, applyTheme);
  media?.addEventListener("change", applyTheme);
}

export function update<K extends keyof SettingsState>(key: K, value: SettingsState[K]): void {
  settings[key] = value;
}
