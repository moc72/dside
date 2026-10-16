import { computed } from "vue";

import { settings } from "../stores/settings";

/**
 * What "everyday" means, in one place. Everyday work is documents and the web, not a codebase:
 * the partner gets no shell, no code review and no sub-agents, the plan/skill/schedule panels
 * are out of the way, and files open as plain text instead of in the code editor.
 *
 * Changing the mode here changes it everywhere — the tools the engine offers, the panels the
 * window shows, and how a file is displayed.
 */
export const EVERYDAY_DISABLED_TOOLS = ["bash", "code_review", "task"] as const;

export const EVERYDAY_HIDDEN_TABS = ["progress", "skills", "schedules"] as const;

export const everydayMode = computed(() => settings.mode === "everyday");

export function toolIsDisabledInThisMode(name: string): boolean {
  return everydayMode.value && EVERYDAY_DISABLED_TOOLS.includes(name as never);
}
