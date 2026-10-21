<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, shallowRef, watch } from "vue";
import * as monaco from "../monaco";

const props = withDefaults(
  defineProps<{
    original?: string;
    modified?: string;
    language?: string;
    theme?: string;
  }>(),
  {
    original: "",
    modified: "",
    language: "plaintext",
    theme: "vs-dark",
  },
);

const container = ref<HTMLDivElement | null>(null);
const editor = shallowRef<monaco.editor.IStandaloneDiffEditor | null>(null);
let originalModel: monaco.editor.ITextModel | null = null;
let modifiedModel: monaco.editor.ITextModel | null = null;
let serial = 0;

function setModels(): void {
  const instance = editor.value;
  if (!instance) return;
  serial += 1;
  originalModel?.dispose();
  modifiedModel?.dispose();
  originalModel = monaco.editor.createModel(
    props.original,
    props.language,
    monaco.Uri.parse(`inmemory://diff/${serial}/original`),
  );
  modifiedModel = monaco.editor.createModel(
    props.modified,
    props.language,
    monaco.Uri.parse(`inmemory://diff/${serial}/modified`),
  );
  instance.setModel({ original: originalModel, modified: modifiedModel });
}

onMounted(() => {
  if (!container.value) return;
  monaco.editor.setTheme(props.theme);
  editor.value = monaco.editor.createDiffEditor(container.value, {
    readOnly: true,
    originalEditable: false,
    automaticLayout: true,
    renderSideBySide: true,
    minimap: { enabled: false },
    fontSize: 13,
    scrollBeyondLastLine: false,
    renderOverviewRuler: false,
  });
  setModels();
});

watch(
  () => [props.original, props.modified, props.language],
  () => setModels(),
);

watch(
  () => props.theme,
  (theme) => monaco.editor.setTheme(theme),
);

onBeforeUnmount(() => {
  editor.value?.dispose();
  originalModel?.dispose();
  modifiedModel?.dispose();
  originalModel = null;
  modifiedModel = null;
});
</script>

<template>
  <div ref="container" class="monaco-diff"></div>
</template>

<style scoped>
.monaco-diff {
  width: 100%;
  height: 100%;
  min-height: 0;
}
</style>
