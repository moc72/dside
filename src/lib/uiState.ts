import { invoke } from "@tauri-apps/api/core";

/**
 * Window state that is worth remembering but is not a preference: which session was open,
 * which tabs were on the work area, which file was last previewed. It shares the `settings`
 * table with a `ui.` prefix — one storage layer, no second file to keep in sync.
 */
const PREFIX = "ui.";

const pending = new Map<string, unknown>();
let timer: number | undefined;
let ready = false;

/** Reads the `ui.*` entries so the stores can hydrate before the app mounts. */
export async function loadUiState(): Promise<Record<string, unknown>> {
  const state: Record<string, unknown> = {};
  try {
    const stored = await invoke<Record<string, string>>("settings_all");
    for (const [key, raw] of Object.entries(stored)) {
      if (!key.startsWith(PREFIX)) continue;
      try {
        state[key.slice(PREFIX.length)] = JSON.parse(raw);
      } catch {
        // unreadable value — fall back to the default rather than fail startup
      }
    }
  } catch {
    // no database yet — the defaults are good enough
  }
  ready = true;
  return state;
}

function flush(): void {
  const entries = [...pending.entries()];
  pending.clear();
  for (const [key, value] of entries) {
    void invoke("settings_set", { key, value: JSON.stringify(value) }).catch(() => undefined);
  }
}

/** Best effort and debounced: remembering the last tab must never hold up a click. */
export function saveUiState(key: string, value: unknown): void {
  if (!ready) return;
  pending.set(PREFIX + key, value);
  if (timer) window.clearTimeout(timer);
  timer = window.setTimeout(() => {
    timer = undefined;
    flush();
  }, 400);
}
