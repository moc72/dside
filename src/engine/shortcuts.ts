import { ref } from "vue";
import type { MessageKey } from "../i18n";
import { settings } from "../stores/settings";

/**
 * One place that knows which commands exist, what they are called and which keys they use.
 * The panel and the global key handler both read from here; a binding the user changes is
 * stored as an override in `settings.shortcuts` and never mutates the defaults.
 */
export type ShortcutScope = "global" | "input";

export interface ShortcutCommand {
  id: string;
  label: MessageKey;
  group: "general" | "navigation" | "appearance" | "input";
  scope: ShortcutScope;
  defaults: string[];
  /** Fixed commands belong to the composer and are only shown in the panel. */
  fixed?: boolean;
}

export const COMMANDS: ShortcutCommand[] = [
  {
    id: "search",
    label: "shortcuts.commands.search",
    group: "general",
    scope: "global",
    defaults: ["Ctrl+K", "Ctrl+Shift+P"],
  },
  {
    id: "settings",
    label: "shortcuts.commands.settings",
    group: "general",
    scope: "global",
    defaults: ["Ctrl+,"],
  },
  {
    id: "new-session",
    label: "shortcuts.commands.newSession",
    group: "general",
    scope: "global",
    defaults: ["Ctrl+N"],
  },
  {
    id: "toggle-rail",
    label: "shortcuts.commands.toggleRail",
    group: "navigation",
    scope: "global",
    defaults: ["Ctrl+B"],
  },
  {
    id: "toggle-work",
    label: "shortcuts.commands.toggleWork",
    group: "navigation",
    scope: "global",
    defaults: ["Ctrl+Alt+B"],
  },
  {
    id: "work-area",
    label: "shortcuts.commands.workArea",
    group: "navigation",
    scope: "global",
    defaults: ["Ctrl+O"],
  },
  {
    id: "close-tab",
    label: "shortcuts.commands.closeTab",
    group: "navigation",
    scope: "global",
    defaults: ["Ctrl+W"],
  },
  {
    id: "prev-session",
    label: "shortcuts.commands.prevSession",
    group: "navigation",
    scope: "global",
    defaults: ["Alt+ArrowUp"],
  },
  {
    id: "next-session",
    label: "shortcuts.commands.nextSession",
    group: "navigation",
    scope: "global",
    defaults: ["Alt+ArrowDown"],
  },
  {
    id: "toggle-mode",
    label: "shortcuts.commands.toggleMode",
    group: "appearance",
    scope: "global",
    defaults: ["Ctrl+Shift+M"],
  },
  {
    id: "cycle-effort",
    label: "shortcuts.commands.cycleEffort",
    group: "appearance",
    scope: "global",
    defaults: ["Ctrl+T"],
  },
  {
    id: "toggle-theme",
    label: "shortcuts.commands.toggleTheme",
    group: "appearance",
    scope: "global",
    defaults: ["Ctrl+Shift+L"],
  },
  {
    id: "send",
    label: "shortcuts.commands.send",
    group: "input",
    scope: "input",
    defaults: ["Enter"],
    fixed: true,
  },
  {
    id: "newline",
    label: "shortcuts.commands.newline",
    group: "input",
    scope: "input",
    defaults: ["Shift+Enter"],
    fixed: true,
  },
];

/** The panel sets this while it waits for a key, so typing a chord does not run a command. */
export const shortcutCapturing = ref(false);

/** Punctuation that changes when Shift is held; key events report the shifted character. */
const UNSHIFT: Record<string, string> = {
  "{": "[",
  "}": "]",
  ":": ";",
  '"': "'",
  "<": ",",
  ">": ".",
  "?": "/",
  "+": "=",
  _: "-",
  "~": "`",
  "!": "1",
  "@": "2",
  "#": "3",
  $: "4",
  "%": "5",
  "^": "6",
  "&": "7",
  "*": "8",
  "(": "9",
  ")": "0",
};

const MODIFIER_KEYS = new Set(["Control", "Shift", "Alt", "Meta", "OS", "AltGraph", "CapsLock"]);

/**
 * Canonical chord from a key event: Ctrl, Alt, Shift, key. Cmd counts as Ctrl so the same
 * defaults work on macOS; the panel says so.
 */
export function bindingFor(event: KeyboardEvent): string | null {
  if (MODIFIER_KEYS.has(event.key)) return null;
  const parts: string[] = [];
  if (event.ctrlKey || event.metaKey) parts.push("Ctrl");
  if (event.altKey) parts.push("Alt");
  if (event.shiftKey) parts.push("Shift");

  const key = keyName(event);
  if (!key) return null;
  parts.push(key);
  return parts.join("+");
}

function keyName(event: KeyboardEvent): string | null {
  const raw = event.key;
  if (!raw) return null;
  if (raw === " ") return "Space";
  if (raw.length === 1) {
    if (/[a-z]/i.test(raw)) return raw.toUpperCase();
    return UNSHIFT[raw] ?? raw;
  }
  return raw;
}

/** A global shortcut must carry Ctrl/Alt/Cmd; otherwise typing would trigger it. */
export function isSafeGlobalBinding(binding: string): boolean {
  const parts = binding.split("+");
  return parts.includes("Ctrl") || parts.includes("Alt") || parts.includes("Meta");
}

export function commandById(id: string): ShortcutCommand | undefined {
  return COMMANDS.find((command) => command.id === id);
}

export function activeBindings(id: string): string[] {
  const stored = settings.shortcuts?.[id];
  if (Array.isArray(stored)) {
    return stored.filter((binding): binding is string => typeof binding === "string");
  }
  return commandById(id)?.defaults ?? [];
}

/** The command that already owns a chord, if any. */
export function bindingOwner(binding: string, exceptId?: string): ShortcutCommand | undefined {
  return COMMANDS.find(
    (command) =>
      command.id !== exceptId && !command.fixed && activeBindings(command.id).includes(binding),
  );
}

export function setBindings(id: string, bindings: string[]): void {
  settings.shortcuts = { ...settings.shortcuts, [id]: bindings };
}

export function resetCommand(id: string): void {
  const next = { ...settings.shortcuts };
  delete next[id];
  settings.shortcuts = next;
}

export function resetAllBindings(): void {
  settings.shortcuts = {};
}
