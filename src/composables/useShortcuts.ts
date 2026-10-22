import { onBeforeUnmount, onMounted } from "vue";
import {
  activeBindings,
  bindingFor,
  COMMANDS,
  shortcutCapturing,
} from "../engine/shortcuts";
import { effortsFor } from "../stores/models";
import { activeSessionId, sessions } from "../stores/sessions";
import { firstRun, resolvedTheme, settings } from "../stores/settings";
import { closeSearch, openSearch, searchOpen, settingsItem, toggleRail, view } from "../stores/ui";
import { activeTabId, closeTab, showWorkArea, toggleWorkArea } from "../stores/workArea";
import { createSession, openSession } from "./useEngine";

/**
 * One global key listener for everything in the shortcut registry. It runs in the bubble
 * phase, so a dialog or the composer can handle a key first; `shortcutCapturing` pauses it
 * while the panel waits for a new chord.
 */
export function useShortcuts(enabled: () => boolean = () => true): void {
  const onKey = (event: KeyboardEvent): void => {
    if (!enabled() || shortcutCapturing.value || firstRun.value) return;
    if (event.defaultPrevented || event.repeat) return;

    // The shell and the commit field own their keys: Ctrl+B/N/O/W are line editing there,
    // not app commands. Clicking anywhere else restores the global bindings.
    const target = event.target;
    if (
      target instanceof Element &&
      target.closest("[data-terminal-input], [data-shortcut-passthrough]")
    ) {
      return;
    }

    if (event.key === "Escape" && searchOpen.value) {
      closeSearch();
      return;
    }

    const binding = bindingFor(event);
    if (!binding) return;

    const command = COMMANDS.find(
      (entry) => entry.scope === "global" && activeBindings(entry.id).includes(binding),
    );
    if (!command) return;

    event.preventDefault();
    event.stopPropagation();
    run(command.id);
  };

  onMounted(() => window.addEventListener("keydown", onKey));
  onBeforeUnmount(() => window.removeEventListener("keydown", onKey));
}

function run(id: string): void {
  switch (id) {
    case "search":
      openSearch();
      break;
    case "settings":
      view.value = "settings";
      settingsItem.value = "general";
      break;
    case "new-session":
      void createSession();
      break;
    case "prev-session":
      stepSession(-1);
      break;
    case "next-session":
      stepSession(1);
      break;
    case "toggle-rail":
      toggleRail();
      break;
    case "toggle-work":
      toggleWorkArea();
      break;
    case "work-area":
      showWorkArea();
      break;
    case "close-tab":
      if (activeTabId.value) closeTab(activeTabId.value);
      break;
    case "toggle-mode":
      settings.mode = settings.mode === "coding" ? "everyday" : "coding";
      break;
    case "cycle-effort":
      cycleEffort();
      break;
    case "toggle-theme":
      settings.appearance = resolvedTheme.value === "dark" ? "light" : "dark";
      break;
  }
}

function stepSession(direction: number): void {
  const list = sessions.value;
  if (list.length < 2) return;
  const index = list.findIndex((session) => session.id === activeSessionId.value);
  if (index < 0) return;
  const next = list[(index + direction + list.length) % list.length];
  if (next) void openSession(next.id);
}

/**
 * Empty means the provider default. From there the cycle walks the efforts the model
 * actually reports, then back to the default.
 */
function cycleEffort(): void {
  const levels = effortsFor(settings.model);
  if (levels.length === 0) return;
  const index = levels.indexOf(settings.reasoningEffort);
  settings.reasoningEffort = index < 0 ? (levels[0] ?? "") : (levels[index + 1] ?? "");
}
