import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

/**
 * Model ids the provider reports for the stored key. Loaded on demand — no hardcoded
 * list, because guessing model names is how you end up with a dropdown full of 404s.
 */
export const knownModels = ref<string[]>([]);
export const modelsError = ref<string | null>(null);

export async function loadModels(): Promise<void> {
  modelsError.value = null;
  try {
    knownModels.value = await invoke<string[]>("engine_models");
  } catch (error) {
    modelsError.value = String(error);
  }
}
