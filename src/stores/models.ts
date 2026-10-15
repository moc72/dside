import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

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
 * Model ids the provider reports for the stored key. The API also knows friendlier names
 * ("DeepSeek-V4.1-Flash" for `deepseek-flash`), so we show those and send the id.
 */
export const knownModels = ref<ModelInfo[]>([]);
export const modelsError = ref<string | null>(null);

export const balance = ref<BalanceInfo | null>(null);
export const balanceError = ref<string | null>(null);

export async function loadModels(): Promise<void> {
  modelsError.value = null;
  try {
    knownModels.value = await invoke<ModelInfo[]>("engine_models");
  } catch (error) {
    modelsError.value = String(error);
  }
}

export async function loadBalance(): Promise<void> {
  balanceError.value = null;
  try {
    balance.value = await invoke<BalanceInfo>("provider_balance");
  } catch (error) {
    balanceError.value = String(error);
  }
}
