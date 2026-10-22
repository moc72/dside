<script setup lang="ts">
import { computed } from "vue";
import ToggleRow from "./ToggleRow.vue";
import { t } from "../../i18n";
import { defaultAllowedCommands, settings } from "../../stores/settings";

/**
 * Guardrails for the companion, not for the user's terminal. The allowlist is a policy:
 * program names, no paths, no shell redirection; it is deliberately not called a sandbox.
 */
const allowlist = computed({
  get: () => settings.commandPolicy === "allowlist",
  set: (value: boolean) => {
    settings.commandPolicy = value ? "allowlist" : "all";
  },
});

const commands = computed({
  get: () => settings.allowedCommands.join("\n"),
  set: (value: string) => {
    const next: string[] = [];
    for (const entry of value.split(/[\n,]+/)) {
      const clean = entry.trim().toLowerCase();
      if (clean && !next.includes(clean)) next.push(clean);
    }
    settings.allowedCommands = next;
  },
});

function resetCommands(): void {
  settings.allowedCommands = [...defaultAllowedCommands];
}
</script>

<template>
  <div class="sandbox">
    <p class="hint">{{ t("sandbox.hint") }}</p>

    <div class="card">
      <ToggleRow
        :model-value="allowlist"
        :label="t('sandbox.allowlist')"
        :description="t('sandbox.allowlistDesc')"
        @update:model-value="allowlist = $event"
      />

      <div class="commands" :class="{ disabled: !allowlist }">
        <div class="commands-head">
          <span class="label">{{ t("sandbox.commands") }}</span>
          <button class="mini-text" type="button" @click="resetCommands">
            {{ t("sandbox.reset") }}
          </button>
        </div>
        <textarea
          v-model="commands"
          class="commands-input"
          rows="9"
          spellcheck="false"
          :disabled="!allowlist"
        ></textarea>
        <p class="hint">{{ t("sandbox.commandsHint") }}</p>
      </div>

      <ToggleRow
        :model-value="settings.workspaceOnly"
        :label="t('sandbox.workspace')"
        :description="t('sandbox.workspaceDesc')"
        @update:model-value="settings.workspaceOnly = $event"
      />
    </div>

    <p class="note">{{ t("sandbox.note") }}</p>
  </div>
</template>

<style scoped>
.sandbox {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.hint {
  margin: 0;
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.5;
}

.card {
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg-panel);
}

.commands {
  padding: 14px 16px;
  border-top: 1px solid var(--border);
}

.commands.disabled {
  opacity: 0.55;
}

.commands-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 8px;
}

.label {
  color: var(--text-strong);
  font-size: 13px;
}

.commands-input {
  width: 100%;
  box-sizing: border-box;
  padding: 8px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-input);
  color: var(--text);
  font-family: var(--mono);
  font-size: 11.5px;
  line-height: 1.5;
  resize: vertical;
}

.commands-input:focus {
  border-color: var(--accent);
  outline: none;
}

.commands .hint {
  margin-top: 6px;
}

.note {
  margin: 0;
  padding: 9px 11px;
  border-left: 2px solid var(--accent);
  background: var(--bg-panel);
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.5;
}
</style>
