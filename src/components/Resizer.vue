<script setup lang="ts">
defineProps<{
  align: "left" | "right";
}>();

const emit = defineEmits<{ start: [event: PointerEvent] }>();

function onPointerDown(event: PointerEvent) {
  emit("start", event);
}
</script>

<template>
  <div class="resizer" :class="align" @pointerdown="onPointerDown"></div>
</template>

<style scoped>
.resizer {
  position: absolute;
  top: 0;
  bottom: 0;
  width: 14px;
  z-index: 6;
  cursor: col-resize;
  touch-action: none;
}

.resizer.left {
  transform: translateX(-50%);
}

.resizer.right {
  transform: translateX(50%);
}

.resizer::before {
  content: "";
  position: absolute;
  top: 0;
  bottom: 0;
  left: 50%;
  width: 2px;
  transform: translateX(-50%);
  background: transparent;
  transition: background 0.12s ease;
}

.resizer:hover::before,
.resizer:active::before {
  background: var(--accent);
}
</style>
