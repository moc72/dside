import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

import { PARTNER_NAME_PLACEHOLDER } from "../engine/names";

/**
 * The partner's own name. `null` means it has not decided yet and still goes by the
 * placeholder; the settings menu shows the placeholder until then and the chosen name after.
 *
 * Choosing happens on the model's side (the `choose_name` tool) and once only, so nothing here
 * offers to write it.
 */
export const partnerName = ref<string | null>(null);

/**
 * What the UI is allowed to show. The name is chosen early, but the user is told it only
 * after enough trust has been earned — knowing it is the reward, not the start.
 */
export const partnerDisplayName = ref<string>(PARTNER_NAME_PLACEHOLDER);

export interface TrustStatus {
  points: number;
  threshold: number;
  revealed: boolean;
  name: string | null;
  breakdown: Array<[string, number]>;
}

export const trustPoints = ref(0);
export const trustThreshold = ref(100);
/** Fail open: a failed read must never hide a name the user has already been shown. */
export const trustRevealed = ref(true);
export const trustBreakdown = ref<Array<[string, number]>>([]);
/** Set when the threshold is crossed while the app is open; the banner shows it once. */
export const partnerReveal = ref<string | null>(null);

export async function refreshPartnerName(): Promise<void> {
  try {
    partnerName.value = await invoke<string | null>("partner_name");
  } catch {
    partnerName.value = null;
  }
  try {
    const status = await invoke<TrustStatus>("trust_status");
    trustPoints.value = status.points;
    trustThreshold.value = status.threshold;
    trustRevealed.value = status.revealed;
    trustBreakdown.value = status.breakdown ?? [];
  } catch {
    trustRevealed.value = true;
  }
  const chosen = partnerName.value?.trim() ?? "";
  partnerDisplayName.value =
    chosen && trustRevealed.value ? chosen : PARTNER_NAME_PLACEHOLDER;
}
