<script setup lang="ts">
import { computed } from "vue";
import { t } from "../../i18n";
import { modelChoices } from "../../stores/models";
import { settings } from "../../stores/settings";
import { lastUsage } from "../../composables/useEngine";

/**
 * How much of the model's context is used, as a ring. The number comes from the provider's
 * own usage report for the last request, the limit from the model list — nothing here is
 * estimated. Clicking it would mean compressing the conversation, which does not exist yet,
 * so it stays a display.
 */
const RADIUS = 7;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

const limit = computed(
  () => modelChoices().find((entry) => entry.id === settings.model)?.contextWindow ?? 0,
);

const share = computed(() => {
  if (!limit.value || !lastUsage.value) return 0;
  return Math.min(1, lastUsage.value.promptTokens / limit.value);
});

const percent = computed(() => Math.round(share.value * 100));

const hint = computed(() => {
  if (!lastUsage.value) return t("composer.contextUnknown");
  if (!limit.value) return t("composer.tokens", { used: lastUsage.value.promptTokens });
  return t("composer.contextHint", {
    used: lastUsage.value.promptTokens,
    total: limit.value,
    percent: percent.value,
  });
});

const colour = computed(() => {
  if (percent.value >= 85) return "var(--danger)";
  if (percent.value >= 60) return "var(--accent)";
  return "var(--text-muted)";
});
</script>

<template>
  <span class="ring" :title="hint">
    <svg viewBox="0 0 18 18" width="18" height="18" aria-hidden="true">
      <circle cx="9" cy="9" :r="RADIUS" fill="none" stroke="var(--border-strong)" stroke-width="2" />
      <circle
        cx="9"
        cy="9"
        :r="RADIUS"
        fill="none"
        :stroke="colour"
        stroke-width="2"
        stroke-linecap="round"
        :stroke-dasharray="CIRCUMFERENCE"
        :stroke-dashoffset="CIRCUMFERENCE * (1 - share)"
        transform="rotate(-90 9 9)"
      />
    </svg>
  </span>
</template>

<style scoped>
.ring {
  display: grid;
  place-items: center;
  width: 24px;
  height: 24px;
  flex: none;
}
</style>
