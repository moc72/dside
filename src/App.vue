<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Resizer from "./components/Resizer.vue";
import TitleBar from "./components/TitleBar.vue";
import LeftRail from "./components/LeftRail.vue";
import SearchDialog from "./components/SearchDialog.vue";
import WebDesignPanel from "./components/WebDesignPanel.vue";
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

/**
 * Launch splash. The bar is theatre — nothing is downloaded — but a bar that moves
 * unevenly reads as "working" while a smooth one reads as "fake". The curve lives in CSS
 * (`splash-progress`), because `requestAnimationFrame` does not fire reliably in a splash
 * window under Wayland: the first version stalled here and never handed over.
 */
const SPLASH_MS = 7000;

/** The splash stays hidden until its artwork is ready, then holds for the full run. */
async function finishSplash() {
  const say = (text: string) => {
    void invoke("log_message", { text }).catch(() => undefined);
  };

  say("splash: waiting for artwork");
  await new Promise<void>((resolve) => {
    const image = new Image();
    image.onload = () => resolve();
    image.onerror = () => resolve();
    image.src = "/whale-splash.png";
  });
  say("splash: artwork ready");

  await invoke("splash_visible").catch(() => undefined);
  say("splash: visible");

  await new Promise<void>((resolve) => {
    window.setTimeout(resolve, SPLASH_MS);
  });
  say("splash: progress done");

  await invoke("splash_ready").catch(() => undefined);
  say("splash: handed over");
}

onMounted(() => {
  if (isSplash.value) {
    void finishSplash();
    return;
  }
  void invoke("log_message", { text: "main window: mounted" }).catch(() => undefined);
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
    <div class="splash-disc">
      <img src="/whale-splash.png" class="splash-whale" alt="" />
      <span class="splash-mark" v-html="deepSeekMark"></span>
      <div class="splash-bar"><span></span></div>
    </div>
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
    <WebDesignPanel />
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
/**
 * The splash draws a disc on a transparent window, so what appears is a circle rather than a
 * rectangle sitting in the middle of the screen. If the compositor refuses transparency the
 * disc still reads as a disc — a dark square behind it is the worst case.
 */
.splash-window {
  display: grid;
  place-items: center;
  height: 100vh;
  background: transparent;
}

.splash-disc {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 14px;
  width: 300px;
  height: 300px;
  border: 1px solid var(--border-strong);
  border-radius: 50%;
  background: var(--bg-app);
  box-shadow: 0 18px 60px rgb(0 0 0 / 45%);
  overflow: hidden;
}

.splash-disc .splash-whale {
  width: 150px;
  height: auto;
  position: relative;
  z-index: 0;
  /* The whale grows towards the edge of the disc while the progress bar runs — the same 7
     seconds, so the two read as one movement. The disc clips it. */
  animation: whale-zoom 7s linear forwards;
  transform-origin: 50% 55%;
  will-change: transform;
}

@keyframes whale-zoom {
  from {
    transform: scale(1);
  }

  to {
    transform: scale(2.05);
  }
}

/* Text and bar stay above the whale, whatever size it reaches. */
.splash-mark,
.splash-bar {
  position: relative;
  z-index: 1;
}

@media (prefers-reduced-motion: reduce) {
  .splash-disc .splash-whale {
    animation: none;
  }
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

.splash-bar {
  width: 170px;
  height: 3px;
  border-radius: 999px;
  background: var(--bg-elevated);
  overflow: hidden;
}

.splash-bar span {
  display: block;
  height: 100%;
  background: var(--accent);
  border-radius: 999px;
  animation: splash-progress 7s linear forwards;
}

/* Uneven on purpose: plateaus read as work, jumps as progress. */
@keyframes splash-progress {
  0% {
    width: 0%;
  }
  6% {
    width: 7%;
  }
  11% {
    width: 8%;
  }
  18% {
    width: 24%;
  }
  22% {
    width: 25%;
  }
  31% {
    width: 46%;
  }
  38% {
    width: 50%;
  }
  49% {
    width: 69%;
  }
  57% {
    width: 71%;
  }
  67% {
    width: 86%;
  }
  75% {
    width: 87%;
  }
  85% {
    width: 96%;
  }
  93% {
    width: 99%;
  }
  100% {
    width: 100%;
  }
}
</style>
