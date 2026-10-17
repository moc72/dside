<script setup lang="ts">
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import Icon from "../Icon.vue";
import { t } from "../../i18n";
import { links } from "../../engine/links";

/**
 * The About page. Version and package list come from the manifests themselves (the Rust side
 * embeds both at compile time), so this page cannot drift away from what is actually built.
 */
interface AboutPackage {
  name: string;
  version: string;
}

interface AboutInfo {
  version: string;
  frontend: AboutPackage[];
  backend: AboutPackage[];
}

const info = ref<AboutInfo | null>(null);
const openPanel = ref<"frontend" | "backend" | null>(null);

const authors = ["Ziggy Shēn", "Modest Coder"];

async function openLink(url: string) {
  try {
    if (/^https?:/i.test(url)) await invoke("open_browser_window", { url });
    else await openUrl(url);
  } catch {
    // nothing sensible to do if even that fails
  }
}

onMounted(async () => {
  try {
    info.value = await invoke<AboutInfo>("about_info");
  } catch {
    info.value = null;
  }
});
</script>

<template>
  <h1 class="title">{{ t("about.title") }}</h1>
  <p class="hint">{{ t("about.tagline") }}</p>

  <h2 class="section">{{ t("about.version") }}</h2>
  <div class="card">
    <div class="row">
      <div class="text">
        <div class="label">DS·IDE</div>
        <div class="desc">{{ t("about.versionValue", { version: info?.version ?? "—" }) }}</div>
      </div>
    </div>
    <div class="row">
      <div class="text">
        <div class="label">{{ t("about.authors") }}</div>
        <div class="desc">{{ authors.join(" · ") }}</div>
      </div>
    </div>
    <div class="row">
      <div class="text">
        <div class="label">{{ t("about.blueprint") }}</div>
        <div class="desc">{{ t("about.blueprintValue") }}</div>
      </div>
    </div>
    <div class="row">
      <div class="text">
        <div class="label">{{ t("about.license") }}</div>
        <div class="desc">{{ t("about.licenseValue") }}</div>
      </div>
    </div>
    <div class="row">
      <div class="text">
        <div class="label">{{ t("about.fonts") }}</div>
        <div class="desc">{{ t("about.fontsValue") }}</div>
      </div>
    </div>
  </div>

  <h2 class="section">{{ t("about.links") }}</h2>
  <div class="card">
    <button class="row link" type="button" @click="openLink(links.home)">
      <div class="text">
        <div class="label">{{ t("about.homepage") }}</div>
        <div class="desc">{{ links.home }}</div>
      </div>
      <Icon name="external-link" :size="14" />
    </button>
    <button class="row link" type="button" @click="openLink(links.github)">
      <div class="text">
        <div class="label">{{ t("about.source") }}</div>
        <div class="desc">{{ links.github }}</div>
      </div>
      <Icon name="external-link" :size="14" />
    </button>
    <button class="row link" type="button" @click="openLink(links.mail)">
      <div class="text">
        <div class="label">{{ t("about.contact") }}</div>
        <div class="desc">{{ links.mail.replace("mailto:", "") }}</div>
      </div>
      <Icon name="external-link" :size="14" />
    </button>
  </div>

  <h2 class="section">{{ t("about.packages") }}</h2>
  <div class="card">
    <button class="row link" type="button" @click="openPanel = openPanel === 'frontend' ? null : 'frontend'">
      <div class="text">
        <div class="label">{{ t("about.packagesFrontend") }}</div>
        <div class="desc">{{ t("about.packagesCount", { count: info?.frontend.length ?? 0 }) }}</div>
      </div>
      <Icon :name="openPanel === 'frontend' ? 'chevron-down' : 'chevron-right'" :size="14" />
    </button>
    <ul v-if="openPanel === 'frontend'" class="packages">
      <li v-for="entry in info?.frontend ?? []" :key="entry.name">
        <span class="name">{{ entry.name }}</span>
        <span class="version">{{ entry.version }}</span>
      </li>
    </ul>

    <button class="row link" type="button" @click="openPanel = openPanel === 'backend' ? null : 'backend'">
      <div class="text">
        <div class="label">{{ t("about.packagesBackend") }}</div>
        <div class="desc">{{ t("about.packagesCount", { count: info?.backend.length ?? 0 }) }}</div>
      </div>
      <Icon :name="openPanel === 'backend' ? 'chevron-down' : 'chevron-right'" :size="14" />
    </button>
    <ul v-if="openPanel === 'backend'" class="packages">
      <li v-for="entry in info?.backend ?? []" :key="entry.name">
        <span class="name">{{ entry.name }}</span>
        <span class="version">{{ entry.version }}</span>
      </li>
    </ul>
  </div>
  <p class="hint">{{ t("about.licenses") }}</p>
</template>

<style scoped>
.row.link {
  align-items: center;
  width: 100%;
  border: none;
  border-top: 1px solid var(--border);
  background: transparent;
  color: inherit;
  text-align: left;
  cursor: pointer;
}

.card > .row.link:first-child {
  border-top: none;
}

.row.link:hover {
  background: var(--bg-hover);
}

.packages {
  list-style: none;
  margin: 0;
  padding: 4px 16px 12px;
  border-top: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.packages li {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  color: var(--text-muted);
  font-family: var(--mono);
  font-size: 11px;
}

.packages .version {
  flex: none;
}
</style>
