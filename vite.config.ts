import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
// @ts-expect-error type error without @types/node package
import process from "node:process";
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(() => ({
  plugins: [vue()],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // only scan our own entry, not the unpacked monaco-editor source tree
  optimizeDeps: {
    entries: ["index.html"],
  },
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri` and the monaco source tree
      ignored: ["**/src-tauri/**", "**/monaco-editor-0.57.0/**"],
    },
    // The Tauri webview caches aggressively and keeps deleted modules in its graph,
    // which turns a removed file into a dev-server error on every start.
    headers: { "Cache-Control": "no-store" },
  },
}));
