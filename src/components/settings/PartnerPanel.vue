<script setup lang="ts">
import { computed } from "vue";
import Icon from "../Icon.vue";
import { t } from "../../i18n";
import { PARTNER_NAME_CHOICES, PARTNER_NAME_PLACEHOLDER } from "../../engine/names";
import { locale } from "../../i18n";
import { partnerDisplayName, partnerName } from "../../stores/partner";
import { settings } from "../../stores/settings";

/**
 * Where the user says what their partner should keep in mind. The name is deliberately not
 * editable here: the partner chooses it itself, once, and that is the point of it.
 */
const chosen = computed(() => partnerName.value?.trim() ?? "");

/**
 * What the chosen name means, when we know it. The pool itself is the partner's own reference
 * (`name_ideas`) and is deliberately not shown here — the name is its decision, not a vote.
 */
const chosenMeaning = computed(() => {
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
          <template v-if="chosen">{{ chosen }}</template>
          <template v-else>{{ PARTNER_NAME_PLACEHOLDER }} <span class="soft">{{ t("partner.placeholder") }}</span></template>
        </div>
        <div class="desc">{{ chosen ? t("partner.nameChosen") : t("partner.nameOpen") }}</div>
        <div v-if="chosenMeaning" class="desc mean">{{ chosenMeaning }}</div>
      </div>
      <Icon name="sparkle" :size="16" />
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
