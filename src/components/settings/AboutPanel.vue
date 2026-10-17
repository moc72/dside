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
  <div class="aboutpage">

    <h1 class="title">{{ t("about.title") }}</h1>
    <p class="hint">{{ t("about.tagline") }}</p>
    
    <h2 class="section">{{ t("about.version") }}</h2>

    <div class="row">
      <div class="col">
        <div class="card" style="">
          <div class="row">
            <div class="text">
              <div class="label">DS·IDE</div>
              <div class="desc larger">{{ info?.version ?? "—" }}</div>
            </div>
          </div>
        </div>
        <div class="card">
          <div class="row">
            <div class="text">
              <div class="label">{{ t("about.authors") }}</div>
              <div class="desc larger">{{ authors.join(" · ") }}</div>
            </div>
          </div>
        </div>
      </div>
      <div class="col">
        <div class="card">
          <div class="row">
            <div class="text">
              <div class="label">{{ t("about.blueprint") }}</div>
              <div class="desc larger">{{ t("about.blueprintValue") }}</div>
            </div>
          </div>
          <hr />
          <div class="row">
            <div class="text">
              <div class="label">{{ t("about.license") }}</div>
              <div class="desc larger">{{ t("about.licenseValue") }}</div>
            </div>
          </div>
          <hr />
          <div class="row">
            <div class="text">
              <div class="label">{{ t("about.fonts") }}</div>
              <div class="desc larger">{{ t("about.fontsValue") }}</div>
            </div>
          </div>
        </div>
      </div>
    </div>
    
    <div class="row">
      <div class="col">
        <h2 class="section">{{ t("about.links") }}</h2>
        <div class="card">
          <button class="row link" type="button" @click="openLink(links.home)">
            <div class="text">
              <div class="label">{{ t("about.homepage") }}</div>
              <div class="desc larger">{{ links.home }}</div>
            </div>
            <Icon name="external-link" :size="14" />
          </button>
          <button class="row link" type="button" @click="openLink(links.github)">
            <div class="text">
              <div class="label">{{ t("about.source") }}</div>
              <div class="desc larger">{{ links.github }}</div>
            </div>
            <Icon name="external-link" :size="14" />
          </button>
          <button class="row link" type="button" @click="openLink(links.mail)">
            <div class="text">
              <div class="label">{{ t("about.contact") }}</div>
              <div class="desc larger">{{ links.mail.replace("mailto:", "") }}</div>
            </div>
            <Icon name="external-link" :size="14" />
          </button>
        </div>
      </div>
    </div>

    <div class="row">
      <div class="col">
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
      </div>
      <div class="col">
        <h2 class="section">{{ t("about.builtWith") }}</h2>
        <div class="card">
          <div class="buildgrid">
            <figure class="build">
              <img class="logo" src="/logos/tauri.svg" alt="Tauri" />
              <figcaption>Tauri</figcaption>
            </figure>
            <figure class="build">
              <img class="logo" src="/logos/rust.webp" alt="Rust" />
              <figcaption>Rust</figcaption>
            </figure>
            <figure class="build">
              <img class="logo" src="/logos/vue3.webp" alt="Vue 3" />
              <figcaption>Vue 3</figcaption>
            </figure>
            <figure class="build">
              <img class="logo" src="/logos/vite.svg" alt="Vite" />
              <figcaption>Vite</figcaption>
            </figure>
          </div>
        </div>

      </div>
    </div>
    <p class="hint">{{ t("about.licenses") }}</p>
  </div>
  </template>

<style scoped>

/* The scroll container is the settings content, not this page: an element with auto height
   cannot scroll. Kept as a plain block so the page measures its content. */
.aboutpage {
  max-width: 1100px;
}

.col {
  flex: 1 1 380px;
  min-width: 300px;
  box-sizing: border-box;
  padding: 0 8px;
}

.desc.larger{
   font-size: larger;
}

hr {
  margin: 14px 0;
  border: none;
  border-top: 1px solid var(--border);
}

.logo {
  width: 56px;
  height: 56px;
  object-fit: contain;
}

.buildgrid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 14px;
  padding: 6px 0;
}

.build {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  margin: 0;
}

.build figcaption {
  color: var(--text-muted);
  font-size: 11px;
}

.row {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  gap: 10px;
}

.row.link {
  align-items: center;
  width: 100%;
  padding: 11px 0;
  border: none;
  border-top: 1px solid var(--border);
  background: transparent;
  color: inherit;
  text-align: left;
  cursor: pointer;
}

.card {
  width: 100%;
  margin: 10px 0;
  padding: 16px 22px;
  border-radius: var(--radius);
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
