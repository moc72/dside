<script setup lang="ts">
import { computed } from "vue";
import {
  WHALE_VIEW_BOX,
  whaleBody,
  whaleFin,
  whaleFinPivot,
  whaleTail,
  whaleTailPivot,
} from "./deepseek-whale";

/**
 * The DeepSeek whale, swimming. Three paths with the same fill: the body, the fluke and the
 * pectoral fin. The fluke turns around the notch and the fin around its root, which is only
 * visible where they leave the silhouette — inside it they are the same blue as the body.
 */
const props = withDefaults(
  defineProps<{
    /** Rendered height in pixels; the width follows the mark's proportions. */
    height?: number;
    /** No movement — for print, for a still that sits next to other stills, or reduced motion. */
    still?: boolean;
    /** Accessible name; empty means decorative. */
    label?: string;
  }>(),
  { height: 32, still: false, label: "" },
);

const style = computed(() => ({
  height: `${props.height}px`,
  "--tail-x": `${whaleTailPivot.x}%`,
  "--tail-y": `${whaleTailPivot.y}%`,
  "--fin-x": `${whaleFinPivot.x}%`,
  "--fin-y": `${whaleFinPivot.y}%`,
}));
</script>

<template>
  <span class="whale" :class="{ still }" :style="style" role="img" :aria-label="label || undefined">
    <svg :viewBox="WHALE_VIEW_BOX" preserveAspectRatio="xMidYMid meet" aria-hidden="true">
      <g class="swim">
        <path class="body" :d="whaleBody" fill-rule="evenodd" />
        <path class="tail" :d="whaleTail" />
        <path class="fin" :d="whaleFin" />
      </g>
    </svg>
  </span>
</template>

<style scoped>
.whale {
  display: inline-block;
  line-height: 0;
  flex: none;
}

/* `overflow: visible`: a turning fluke at the edge of the view box must not be clipped. */
.whale svg {
  display: block;
  width: auto;
  height: 100%;
  overflow: visible;
}

.body,
.tail,
.fin {
  fill: var(--whale-fill, #4d6bfe);
}

/* fill-box makes the percentages refer to the path's own bounding box, which is where the
   pivots were measured from — not to the whole view box (transform-origin: 0 0 there). */
.tail {
  transform-box: fill-box;
  transform-origin: var(--tail-x) var(--tail-y);
  animation: whale-tail 3.6s ease-in-out infinite;
}

.fin {
  transform-box: fill-box;
  transform-origin: var(--fin-x) var(--fin-y);
  animation: whale-fin 2.7s ease-in-out infinite;
}

.swim {
  animation: whale-float 6.4s ease-in-out infinite;
}

@keyframes whale-tail {
  0%,
  100% {
    transform: rotate(-4.5deg);
  }

  50% {
    transform: rotate(4.5deg);
  }
}

/* The fin answers the tail a little late — that is what reads as swimming rather than
   as two independent parts. */
@keyframes whale-fin {
  0%,
  100% {
    transform: rotate(3.4deg);
  }

  55% {
    transform: rotate(-3.4deg);
  }
}

@keyframes whale-float {
  0%,
  100% {
    transform: translateY(0);
  }

  50% {
    transform: translateY(-1.6%);
  }
}

.still .tail,
.still .fin,
.still .swim {
  animation: none;
}

@media (prefers-reduced-motion: reduce) {
  .tail,
  .fin,
  .swim {
    animation: none;
  }
}
</style>
