<script setup lang="ts">
import { computed, ref } from "vue";
import Resizer from "./components/Resizer.vue";
import TitleBar from "./components/TitleBar.vue";
import LeftRail from "./components/LeftRail.vue";
import AgentPanel from "./components/agent/AgentPanel.vue";
import WorkArea from "./components/workarea/WorkArea.vue";
import SettingsPage from "./components/settings/SettingsPage.vue";
import {
  CENTER_MIN,
  railCollapsed,
  railWidth,
  setRailWidth,
  setWorkWidth,
  view,
  workWidth,
} from "./stores/ui";
import { visible as workAreaVisible } from "./stores/workArea";

const body = ref<HTMLElement | null>(null);

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
</style>
