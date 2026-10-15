<script setup lang="ts">
defineProps<{
  label: string;
  description?: string;
  modelValue: boolean;
}>();

const emit = defineEmits<{ "update:modelValue": [value: boolean] }>();
</script>

<template>
  <div class="row">
    <div class="text">
      <div class="label">{{ label }}</div>
      <div v-if="description" class="desc">{{ description }}</div>
    </div>
    <button
      class="switch"
      :class="{ on: modelValue }"
      type="button"
      role="switch"
      :aria-checked="modelValue"
      @click="emit('update:modelValue', !modelValue)"
    >
      <span class="knob"></span>
    </button>
  </div>
</template>

<style scoped>
.row {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 14px 16px;
  border-top: 1px solid var(--border);
}

.text {
  flex: 1;
  min-width: 0;
}

.label {
  color: var(--text-strong);
  font-size: 13px;
}

.desc {
  margin-top: 2px;
  color: var(--text-muted);
  font-size: 12px;
}

.switch {
  position: relative;
  flex: none;
  width: 40px;
  height: 22px;
  border: none;
  border-radius: 11px;
  background: var(--bg-active);
  transition: background 0.15s ease;
}

.switch.on {
  background: var(--accent-strong);
}

.knob {
  position: absolute;
  top: 3px;
  left: 3px;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--text-strong);
  transition: transform 0.15s ease;
}

.switch.on .knob {
  transform: translateX(18px);
  background: var(--accent-contrast);
}
</style>
