<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, shallowRef, watch } from "vue";
import * as monaco from "../monaco";

const props = withDefaults(
  defineProps<{
    path?: string | null;
    modelValue?: string;
    language?: string;
    theme?: string;
    options?: monaco.editor.IStandaloneEditorConstructionOptions;
  }>(),
  {
    path: null,
    modelValue: "",
    language: "plaintext",
    theme: "vs-dark",
    options: () => ({}),
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: string];
  ready: [editor: monaco.editor.IStandaloneCodeEditor];
}>();

const container = ref<HTMLDivElement | null>(null);
const editor = shallowRef<monaco.editor.IStandaloneCodeEditor | null>(null);
const models = new Map<string, monaco.editor.ITextModel>();

function modelFor(path: string, value: string, language: string) {
  const existing = models.get(path);
  if (existing && !existing.isDisposed()) {
    if (existing.getValue() !== value) existing.setValue(value);
    if (existing.getLanguageId() !== language) monaco.editor.setModelLanguage(existing, language);
    return existing;
  }

  const model = monaco.editor.createModel(value, language, monaco.Uri.parse(`inmemory://ide/${path}`));
  model.onDidChangeContent(() => {
    if (editor.value?.getModel() === model) emit("update:modelValue", model.getValue());
  });
  models.set(path, model);
  return model;
}

function applyPath() {
  const instance = editor.value;
  if (!instance) return;
  if (!props.path) {
    instance.setModel(null);
    return;
  }
  instance.setModel(modelFor(props.path, props.modelValue, props.language));
}

onMounted(() => {
  if (!container.value) return;

  editor.value = monaco.editor.create(container.value, {
    theme: props.theme,
    automaticLayout: true,
    fontSize: 14,
    minimap: { enabled: true },
    scrollBeyondLastLine: false,
    ...props.options,
  });

  applyPath();
  emit("ready", editor.value);
});

watch(
  () => props.path,
  () => applyPath(),
);

watch(
  () => props.modelValue,
  (value) => {
    const model = editor.value?.getModel();
    if (model && !model.isDisposed() && model.getValue() !== value) model.setValue(value);
  },
);

watch(
  () => props.language,
  (language) => {
    const model = editor.value?.getModel();
    if (model && !model.isDisposed()) monaco.editor.setModelLanguage(model, language);
  },
);

watch(
  () => props.theme,
  (theme) => monaco.editor.setTheme(theme),
);

onBeforeUnmount(() => {
  editor.value?.dispose();
  models.forEach((model) => model.dispose());
  models.clear();
});
</script>

<template>
  <div ref="container" class="monaco-editor"></div>
</template>

<style scoped>
.monaco-editor {
  width: 100%;
  height: 100%;
  min-height: 0;
}
</style>
