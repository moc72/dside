<script setup lang="ts">
import { computed } from "vue";
import Icon from "../Icon.vue";
import { locale, t } from "../../i18n";
import type { MessageKey } from "../../i18n";
import { PARTNER_NAME_CHOICES, PARTNER_NAME_PLACEHOLDER } from "../../engine/names";
import {
  partnerDisplayName,
  partnerName,
  trustBreakdown,
  trustPoints,
  trustRevealed,
  trustThreshold,
} from "../../stores/partner";
import { settings } from "../../stores/settings";

/**
 * Where the user says what their partner should keep in mind. The name is deliberately not
 * editable here: the partner chooses it itself, once, and that is the point of it.
 */
const chosen = computed(() => partnerName.value?.trim() ?? "");

const share = computed(() => {
  const threshold = trustThreshold.value || 1;
  return Math.min(100, Math.round((trustPoints.value / threshold) * 100));
});

/** Where the points came from, in the user's words. Unknown kinds are shown as they are. */
const SOURCE_KEYS: Record<string, MessageKey> = {
  day: "partner.trustDay",
  turn: "partner.trustTurn",
  memory: "partner.trustMemory",
  correction: "partner.trustCorrection",
};

const sources = computed(() =>
  trustBreakdown.value.map(([kind, points]) => ({
    kind,
    points,
    label: SOURCE_KEYS[kind] ? t(SOURCE_KEYS[kind]) : kind,
  })),
);

/**
 * What the chosen name means. The pool itself is the partner's own reference (`name_ideas`) and
 * is deliberately not shown here — the name is its decision, not a vote.
 *
 * A chosen name gets its meaning from the language files, so it reads in every language the
 * app speaks. Names we do not have a translation for fall back to what the pool says (German
 * and English), which is better than showing nothing.
 */
const MEANING_KEYS: Record<string, MessageKey> = {
  ziggy: "partner.meaningZiggy",
};

const chosenMeaning = computed(() => {
  // Before the reveal the meaning would give the name away; it waits with it.
  if (!trustRevealed.value) return "";
  const key = MEANING_KEYS[chosen.value.toLowerCase()];
  if (key) return t(key);
  const choice = PARTNER_NAME_CHOICES.find(
    (entry) => entry.name.toLowerCase() === chosen.value.toLowerCase(),
  );
  if (!choice) return "";
  return locale.value === "de" ? choice.de : choice.en;
});
</script>

<template>
  <h1 class="title">{{ partnerDisplayName }}</h1>
  <p class="hint">{{ t("partner.hint") }}</p>

  <h2 class="section">{{ t("partner.name") }}</h2>
  <div class="card">
    <div class="row">
      <div class="text">
        <div class="label">
          <template v-if="chosen && trustRevealed">{{ chosen }}</template>
          <template v-else>{{ PARTNER_NAME_PLACEHOLDER }} <span class="soft">{{ t("partner.placeholder") }}</span></template>
        </div>
        <div class="desc">
          {{ chosen ? (trustRevealed ? t("partner.nameChosen") : t("partner.trustHidden")) : t("partner.nameOpen") }}
        </div>
        <div v-if="chosenMeaning" class="desc mean">{{ chosenMeaning }}</div>
      </div>
      <Icon name="sparkle" :size="16" />
    </div>
  </div>

  <div v-if="!trustRevealed" class="card trust">
    <div class="row">
      <div class="text">
        <div class="label">{{ t("partner.trust") }}: {{ trustPoints }} / {{ trustThreshold }}</div>
        <div class="bar"><span :style="{ width: share + '%' }"></span></div>
        <div class="desc">{{ t("partner.trustHint", { threshold: trustThreshold }) }}</div>
      </div>
    </div>
    <div v-if="sources.length" class="sources">
      <span v-for="source in sources" :key="source.kind">
        {{ source.label }}: {{ source.points }}
      </span>
    </div>
  </div>

  <h2 class="section">{{ t("partner.prompt") }}</h2>
  <div class="card">
    <div class="row column">
      <div class="desc">{{ t("partner.promptDesc") }}</div>
      <textarea
        v-model="settings.partnerPrompt"
        class="prompt"
        rows="6"
        :placeholder="t('partner.promptPlaceholder')"
      ></textarea>
    </div>
  </div>
</template>

<style scoped>
/* These cards are shared with the settings page by look, not by style sheet: its rules are
   scoped and do not reach into a child component, so the panel carries its own copy. Without
   them the name row and the progress bar render as unstyled blocks. */
.title {
  margin: 0 0 22px;
  color: var(--text-strong);
  font-size: 20px;
  font-weight: 600;
}

.section {
  margin: 22px 0 10px;
  color: var(--text-strong);
  font-size: 13px;
  font-weight: 600;
}

.hint {
  margin: 12px 0 0;
  color: var(--text-muted);
  font-size: 12px;
}

.card {
  margin-bottom: 18px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg-panel);
  overflow: hidden;
}

.row {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 14px 16px;
}

.row .text {
  flex: 1;
  min-width: 0;
}

.label {
  color: var(--text-strong);
  font-size: 13px;
}

.desc {
  margin-top: 2px;
  color: var(--text-muted);
  font-size: 12px;
}

.desc.mean {
  font-style: italic;
}

.card.trust {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.bar {
  height: 7px;
  margin: 9px 0;
  border-radius: 999px;
  background: var(--border);
  overflow: hidden;
}

.bar span {
  display: block;
  height: 100%;
  border-radius: 999px;
  background: var(--accent);
  transition: width 0.4s ease;
}

.sources {
  display: flex;
  flex-wrap: wrap;
  gap: 6px 14px;
  color: var(--text-muted);
  font-size: 11px;
}

.soft {
  color: var(--text-muted);
  font-weight: 400;
}

.row.column {
  align-items: stretch;
  flex-direction: column;
  gap: 8px;
}

.prompt {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  background: var(--bg);
  color: var(--text);
  font-family: inherit;
  font-size: 12px;
  line-height: 1.55;
  resize: vertical;
}

.prompt:focus {
  outline: none;
  border-color: var(--accent);
}
</style>
