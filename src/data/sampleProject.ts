import type { FileNode } from "../types";
import { languageFromPath } from "../lib/language";

function file(name: string, parent: string, content: string): FileNode {
  const path = parent ? `${parent}/${name}` : name;
  return { name, path, isFolder: false, language: languageFromPath(name), content };
}

function folder(name: string, parent: string, children: FileNode[]): FileNode {
  const path = parent ? `${parent}/${name}` : name;
  return { name, path, isFolder: true, expanded: true, children };
}

export const sampleProject: FileNode[] = [
  folder("src", "", [
    file(
      "main.ts",
      "src",
      `import { createApp } from "vue";
import App from "./App.vue";

createApp(App).mount("#app");
`,
    ),
    file(
      "App.vue",
      "src",
      `<script setup lang="ts">
import { ref } from "vue";

const greeting = ref("Hallo aus der eigenen IDE");
</script>

<template>
  <h1>{{ greeting }}</h1>
</template>
`,
    ),
    file(
      "monaco.ts",
      "src",
      `import EditorWorker from "monaco-editor/editor/editor.worker?worker";

self.MonacoEnvironment = {
  getWorker: () => new EditorWorker(),
};
`,
    ),
  ]),
  folder("src-tauri", "", [
    folder("src", "src-tauri", [
      file(
        "main.rs",
        "src-tauri/src",
        `fn main() {
    tauri_app_lib::run();
}
`,
      ),
      file(
        "lib.rs",
        "src-tauri/src",
        `#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
`,
      ),
    ]),
    file(
      "tauri.conf.json",
      "src-tauri",
      `{
  "productName": "tauri-app",
  "identifier": "com.test.monacoeditor"
}
`,
    ),
    file(
      "Cargo.toml",
      "src-tauri",
      `[package]
name = "tauri-app"
version = "0.1.0"
edition = "2024"
`,
    ),
  ]),
  file(
    "README.md",
    "",
    `# Monaco x Tauri

Drei Spalten: Datei-Baum, Editor, Agent-Placeholder.
`,
  ),
  file(
    "package.json",
    "",
    `{
  "name": "tauri-app",
  "private": true,
  "type": "module"
}
`,
  ),
];
