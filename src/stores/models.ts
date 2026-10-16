import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

import { settings, defaultModelFor } from "./settings";

export interface ModelInfo {
  id: string;
  name: string;
  contextWindow?: number | null;
  /** Thinking levels the provider reports for this model. Empty when it does not think. */
  efforts?: string[];
  defaultEffort?: string | null;
}

export interface BalanceInfo {
  available: boolean;
  currency: string;
  total: string;
  granted: string;
  toppedUp: string;
}

/**
 * Model ids the provider reports for the stored key. We send the id and show the name the
 * provider gives it ("DeepSeek-V4.1-Flash" for `deepseek-flash`) — the id alone is what made
 * the field look like it held a wrong value.
 */
export const knownModels = ref<ModelInfo[]>([]);
export const modelsError = ref<string | null>(null);

/**
 * Until `/models` answers. These are the two ids this provider actually has, with the names
 * it gives them — a wrong id here would mean a field that looks fine and fails every request.
 */
const fallbackModels: ModelInfo[] = [
  { id: "deepseek-flash", name: "DeepSeek-V4.1-Flash", efforts: ["low", "high", "max"] },
  { id: "deepseek-v4-pro", name: "DeepSeek-V4-Pro", efforts: ["low", "high", "max"] },
];

export function modelChoices(): ModelInfo[] {
  return knownModels.value.length > 0 ? knownModels.value : fallbackModels;
}

/** The provider's name for a model, or the id itself when it is not in the list. */
export function modelLabel(id: string): string {
  return modelChoices().find((entry) => entry.id === id)?.name ?? id;
}

/** The thinking levels the current model offers; empty means there is nothing to choose. */
export function effortsFor(id: string): string[] {
  return modelChoices().find((entry) => entry.id === id)?.efforts ?? [];
}

export function defaultEffortFor(id: string): string {
  return modelChoices().find((entry) => entry.id === id)?.defaultEffort ?? "";
}

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
