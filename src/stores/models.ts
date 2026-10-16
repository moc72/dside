import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

import { settings, defaultModelFor } from "./settings";

export interface ModelInfo {
  id: string;
  name: string;
  contextWindow?: number | null;
}

export interface BalanceInfo {
  available: boolean;
  currency: string;
  total: string;
  granted: string;
  toppedUp: string;
}

/**
 * Model ids the provider reports for the stored key. The API sends ids only — no display
 * names — so the id is what we show and what we send.
 */
export const knownModels = ref<ModelInfo[]>([]);
export const modelsError = ref<string | null>(null);

export const balance = ref<BalanceInfo | null>(null);
export const balanceError = ref<string | null>(null);

export async function loadModels(): Promise<void> {
  modelsError.value = null;
  try {
    knownModels.value = await invoke<ModelInfo[]>("engine_models");
    repairModelChoice();
  } catch (error) {
    modelsError.value = String(error);
  }
}

/**
 * A model id that the provider does not know means every request fails, and the user sees a
 * field with a name that only looks plausible (an old invented default did exactly that).
 * The choice is corrected once, to the default of the current mode.
 */
function repairModelChoice(): void {
  const ids = knownModels.value.map((entry) => entry.id);
  if (ids.length === 0 || ids.includes(settings.model)) return;
  settings.model = defaultModelFor(settings.mode);
  if (!ids.includes(settings.model)) settings.model = ids[0];
}

export async function loadBalance(): Promise<void> {
  balanceError.value = null;
  try {
    balance.value = await invoke<BalanceInfo>("provider_balance");
  } catch (error) {
    balanceError.value = String(error);
  }
}
