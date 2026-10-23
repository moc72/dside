import { invoke } from "@tauri-apps/api/core";
import { settings } from "../stores/settings";
import { openFile, pulseTab, refreshFile } from "../stores/workArea";

/**
 * Which file each running tool is about. The result event carries no arguments, so the path
 * has to be remembered when the tool starts (see `tool_result` in the engine).
 */
const pendingPaths = new Map<string, string>();

function parseArguments(raw?: string): Record<string, unknown> {
  if (!raw) return {};
  try {
    const value = JSON.parse(raw) as unknown;
    return value && typeof value === "object" ? (value as Record<string, unknown>) : {};
  } catch {
    // A model can emit invalid JSON; there is nothing to follow then.
    return {};
  }
}

function isAbsolute(path: string): boolean {
  return path.startsWith("/") || path.startsWith("\\\\") || /^[A-Za-z]:[\\/]/.test(path);
}

/** The tools address files relative to the workspace, sometimes absolute. */
export function resolveWorkspacePath(path: string): string {
  if (isAbsolute(path)) return path;
  const root = settings.workspace.replace(/[\\/]+$/, "");
  if (!root) return path;
  return `${root}/${path.replace(/^\.\//, "")}`;
}

/**
 * Makes the work area follow what the companion is doing. A file it reads opens at once; a
 * file it writes or edits opens when the result confirms that it was written — before that
 * the file may not exist yet (or the turn may still be waiting for approval). A web fetch
 * moves the browser. `glob`/`grep`/`bash` stay chips: they describe a search, not a document.
 *
 * Deliberately no `focus()` anywhere: the work area changes, but the cursor stays in the
 * composer, so a running turn is not interrupted.
 */
export function followToolStart(id: string, name: string, rawArguments?: string): void {
  const args = parseArguments(rawArguments);

  if (name === "read" || name === "write" || name === "edit") {
    const raw = typeof args.path === "string" ? args.path.trim() : "";
    if (!raw) return;
    const path = resolveWorkspacePath(raw);
    if (name === "read") {
      if (!settings.followCompanion) return;
      openFile(path);
      pulseTab("file");
      return;
    }
    // The tool has not touched the disk yet. Remember the path; the result opens it.
    pendingPaths.set(id, path);
    return;
  }

  if (name === "web_fetch") {
    const url = typeof args.url === "string" ? args.url : "";
    if (!settings.followCompanion || !/^https?:/i.test(url)) return;
    // Reuses the most recent browser window. `focus: false` keeps the main window in front:
    // the page is there when the user looks, but the turn does not steal the keyboard.
    void invoke("open_browser_window", { url, focus: false }).catch(() => undefined);
  }
}

/** A successful write/edit opens the changed file; a failed one leaves the work area alone. */
export function followToolResult(id: string, isError = false): void {
  const path = pendingPaths.get(id);
  if (!path) return;
  pendingPaths.delete(id);
  if (isError) return;
  if (settings.followCompanion) {
    openFile(path);
    pulseTab("file");
  }
  refreshFile(path);
}
