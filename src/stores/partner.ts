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

export const partnerDisplayName = ref<string>(PARTNER_NAME_PLACEHOLDER);

export async function refreshPartnerName(): Promise<void> {
  try {
    partnerName.value = await invoke<string | null>("partner_name");
  } catch {
    partnerName.value = null;
  }
  partnerDisplayName.value = partnerName.value?.trim() || PARTNER_NAME_PLACEHOLDER;
}
