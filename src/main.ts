import { createApp } from "vue";
import App from "./App.vue";
import "./styles/global.css";
import { initReadingSize, initSettings, initTheme } from "./stores/settings";
import { locale } from "./i18n";

initTheme();
initReadingSize();
document.documentElement.lang = locale.value;

// Mount after settings are loaded so theme and language do not flash.
void initSettings()
  .catch(() => undefined)
  .finally(() => {
    createApp(App).mount("#app");
  });
