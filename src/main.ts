import { createApp } from "vue";
import App from "./App.vue";
import "./styles/global.css";
import { initReadingSize, initSettings, initTheme } from "./stores/settings";
import { loadUiState } from "./lib/uiState";
import { initWorkArea } from "./stores/workArea";
import { initSessionSelection } from "./stores/sessions";
import { locale } from "./i18n";

initTheme();
initReadingSize();
document.documentElement.lang = locale.value;

// Mount after settings and window state are loaded, so theme, language, the last session and
// the last previewed file do not flash in one after the other.
async function boot(): Promise<void> {
  await initSettings().catch(() => undefined);
  const ui = await loadUiState();
  initWorkArea(ui.workarea);
  initSessionSelection(ui.activeSession);
}

void boot().finally(() => {
  createApp(App).mount("#app");
});
