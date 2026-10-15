<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import * as pdfjs from "pdfjs-dist";
import workerUrl from "pdfjs-dist/build/pdf.worker.min.mjs?url";
import { t } from "../../i18n";

pdfjs.GlobalWorkerOptions.workerSrc = workerUrl;

const props = defineProps<{ path: string }>();

const host = ref<HTMLDivElement | null>(null);
const pages = ref(0);
const error = ref<string | null>(null);

/** The bytes come from Rust, not from a URL: no fetch, no CORS, no custom-scheme quirks. */
function decodeBase64(value: string): Uint8Array {
  const binary = atob(value);
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index);
  }
  return bytes;
}

async function render(path: string) {
  const container = host.value;
  if (!container) return;

  error.value = null;
  pages.value = 0;
  container.replaceChildren();

  try {
    const data = decodeBase64(await invoke<string>("read_file_base64", { path }));
    const pdf = await pdfjs.getDocument({ data }).promise;
    pages.value = pdf.numPages;

    const available = Math.max(320, container.clientWidth - 32);

    for (let number = 1; number <= pdf.numPages; number += 1) {
      const page = await pdf.getPage(number);
      const base = page.getViewport({ scale: 1 });
      const scale = Math.min(2.5, available / base.width);
      const viewport = page.getViewport({ scale });

      const canvas = window.document.createElement("canvas");
      canvas.width = Math.floor(viewport.width);
      canvas.height = Math.floor(viewport.height);
      canvas.className = "page";
      container.append(canvas);

      await page.render({ canvas, viewport }).promise;
    }
  } catch (cause) {
    error.value = String(cause);
  }
}

onMounted(() => {
  void render(props.path);
});

watch(
  () => props.path,
  (value) => {
    void render(value);
  },
);
</script>

<template>
  <div ref="host" class="pdf">
    <p v-if="error" class="error">{{ t("viewFile.error") }} — {{ error }}</p>
  </div>
</template>

<style scoped>
.pdf {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 16px;
  background: var(--bg-app);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.pdf :deep(.page) {
  max-width: 100%;
  border-radius: 4px;
  background: #ffffff;
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.45);
}

.error {
  margin: 0;
  padding: 12px;
  color: var(--danger);
  font-size: 12px;
  word-break: break-word;
}
</style>
