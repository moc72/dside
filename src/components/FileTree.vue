<script setup lang="ts">
import Icon from "./Icon.vue";
import type { FileNode } from "../types";

withDefaults(
  defineProps<{
    nodes: FileNode[];
    selectedPath?: string | null;
    depth?: number;
  }>(),
  { selectedPath: null, depth: 0 },
);

const emit = defineEmits<{
  select: [node: FileNode];
  toggle: [node: FileNode];
}>();

function activate(node: FileNode) {
  if (node.isFolder) emit("toggle", node);
  else emit("select", node);
}
</script>

<template>
  <ul class="tree">
    <li v-for="node in nodes" :key="node.path">
      <button
        class="row"
        :class="{ selected: node.path === selectedPath }"
        :style="{ paddingLeft: `${6 + depth * 14}px` }"
        type="button"
        @click="activate(node)"
      >
        <Icon
          :name="node.isFolder ? (node.expanded ? 'chevron-down' : 'chevron-right') : 'doc'"
          :size="13"
        />
        <span class="name">{{ node.name }}</span>
      </button>

      <FileTree
        v-if="node.isFolder && node.expanded && node.children && node.children.length"
        :nodes="node.children"
        :selected-path="selectedPath"
        :depth="depth + 1"
        @select="emit('select', $event)"
        @toggle="emit('toggle', $event)"
      />
    </li>
  </ul>
</template>

<style scoped>
.tree {
  list-style: none;
  margin: 0;
  padding: 0;
}

.row {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
  height: 24px;
  padding-right: 8px;
  border: none;
  background: transparent;
  color: var(--text);
  font-size: 12px;
  text-align: left;
  white-space: nowrap;
}

.row:hover {
  background: var(--bg-hover);
}

.row.selected {
  background: var(--accent-strong);
  color: var(--accent-contrast);
}

.name {
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
