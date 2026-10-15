<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Resizer from "./components/Resizer.vue";
import TitleBar from "./components/TitleBar.vue";
import LeftRail from "./components/LeftRail.vue";
import SearchDialog from "./components/SearchDialog.vue";
import AgentPanel from "./components/agent/AgentPanel.vue";
import WorkArea from "./components/workarea/WorkArea.vue";
import SettingsPage from "./components/settings/SettingsPage.vue";
import { deepSeekMark } from "./components/deepseek-mark";
import {
  CENTER_MIN,
  closeSearch,
  openSearch,
  railCollapsed,
  railWidth,
  searchOpen,
  setRailWidth,
  setWorkWidth,
  view,
  workWidth,
} from "./stores/ui";
import { visible as workAreaVisible } from "./stores/workArea";

const body = ref<HTMLElement | null>(null);

/**
 * The splash is a separate, chrome-less Tauri window that loads this same page. It only
 * renders the brand; the main window stays hidden until Rust closes the splash.
 */
let windowLabel = "main";
try {
  windowLabel = getCurrentWindow().label;
} catch {
  // running in a plain browser — treat it as the main window
}
const isSplash = ref(windowLabel === "splash");

/** The splash waits until its artwork is really there, then hands over. */
async function finishSplash() {
  const started = Date.now();
  await new Promise<void>((resolve) => {
    const image = new Image();
    image.onload = () => resolve();
    image.onerror = () => resolve();
    image.src = "/whale-splash.webp";
  });
  // A splash that flashes past is worse than none.
  const remaining = Math.max(0, 1100 - (Date.now() - started));
  window.setTimeout(() => {
    void invoke("splash_ready").catch(() => undefined);
  }, remaining);
}

onMounted(() => {
  if (isSplash.value) {
    void finishSplash();
    return;
  }
  window.addEventListener("keydown", onKey);
});
onBeforeUnmount(() => window.removeEventListener("keydown", onKey));

function onKey(event: KeyboardEvent) {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
    event.preventDefault();
    openSearch();
    return;
  }
  if (event.key === "Escape" && searchOpen.value) closeSearch();
}

const bodyStyle = computed(() => {
  const columns: string[] = [];
  if (!railCollapsed.value) columns.push(`${railWidth.value}%`);
  columns.push("minmax(0, 1fr)");
  if (workAreaVisible.value) columns.push(`${workWidth.value}%`);
  return { gridTemplateColumns: columns.join(" ") };
});

function startDrag(which: "rail" | "work", event: PointerEvent) {
  const container = body.value;
  if (!container) return;
  const total = container.clientWidth;
  if (total <= 0) return;

  const startX = event.clientX;
  const startRail = railWidth.value;
  const startWork = workWidth.value;
  const handle = event.currentTarget as HTMLElement;
  handle.setPointerCapture(event.pointerId);

  const onMove = (move: PointerEvent) => {
    const delta = ((move.clientX - startX) / total) * 100;

    if (which === "rail") {
      const available = 100 - CENTER_MIN - (workAreaVisible.value ? workWidth.value : 0);
      setRailWidth(Math.min(startRail + delta, available));
      return;
    }

    const requested = startWork - delta;
    const railSpace = railCollapsed.value ? 0 : railWidth.value;

    // Free up room by collapsing the left rail when the center would be squeezed.
    if (requested + railSpace + CENTER_MIN > 100 && !railCollapsed.value) {
      railCollapsed.value = true;
    }

    const available = 100 - CENTER_MIN - (railCollapsed.value ? 0 : railWidth.value);
    setWorkWidth(Math.min(requested, available));
  };

  const onUp = () => {
    handle.removeEventListener("pointermove", onMove);
    handle.removeEventListener("pointerup", onUp);
    handle.removeEventListener("pointercancel", onUp);
  };

  handle.addEventListener("pointermove", onMove);
  handle.addEventListener("pointerup", onUp);
  handle.addEventListener("pointercancel", onUp);
}
</script>

<template>
  <div v-if="isSplash" class="splash-window">
    <img src="/whale-splash.webp" alt="" />
    <span class="splash-mark" v-html="deepSeekMark"></span>
  </div>

  <template v-else>
  <SettingsPage v-if="view === 'settings'" @close="view = 'chat'" />

  <div v-else class="ide">
    <TitleBar />
    <div class="stage">
      <div ref="body" class="body" :style="bodyStyle">
        <LeftRail v-if="!railCollapsed" class="card" />
        <AgentPanel class="card" />
        <WorkArea v-if="workAreaVisible" class="card" />

        <Resizer
          v-if="!railCollapsed"
          align="left"
          :style="{ left: `calc(${railWidth}% + 5px)` }"
          @start="(event) => startDrag('rail', event)"
        />
        <Resizer
          v-if="workAreaVisible"
          align="right"
          :style="{ right: `calc(${workWidth}% + 5px)` }"
          @start="(event) => startDrag('work', event)"
        />
      </div>
    </div>
  </div>

  <SearchDialog v-if="searchOpen" />
  </template>
</template>

<style scoped>
.ide {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  background: var(--bg-app);
}

.stage {
  flex: 1;
  min-height: 0;
  display: flex;
  padding: 0 10px 10px;
}

.body {
  position: relative;
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-rows: minmax(0, 1fr);
  gap: 10px;
}

/* The splash window is chrome-less: it shows the brand and nothing else. */
.splash-window {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  height: 100vh;
  background: var(--bg-app);
}

.splash-window img {
  width: 190px;
  height: auto;
}

.splash-mark {
  display: inline-flex;
  color: var(--text-strong);
}

.splash-mark :deep(svg) {
  display: block;
  height: 22px;
  width: auto;
}
</style>
