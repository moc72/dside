<script setup lang="ts">
import { ref } from "vue";

const draft = ref("");
const messages = ref([
  { role: "assistant", text: "Platzhalter. Die echte Kilo-Code-Anbindung folgt später." },
]);

function submit() {
  if (!draft.value.trim()) return;
  messages.value.push({ role: "user", text: draft.value });
  messages.value.push({ role: "assistant", text: "Noch nicht implementiert." });
  draft.value = "";
}
</script>

<template>
  <div class="agent-panel">
    <div class="agent-header">KILO CODE</div>

    <div class="agent-messages">
      <div v-for="(message, index) in messages" :key="index" class="message" :class="message.role">
        {{ message.text }}
      </div>
    </div>

    <form class="agent-input" @submit.prevent="submit">
      <textarea v-model="draft" rows="2" placeholder="Nachricht (noch ohne Wirkung)…"></textarea>
      <button type="submit">Senden</button>
    </form>
  </div>
</template>

<style scoped>
.agent-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  background-color: #252526;
  border-left: 1px solid #3c3c3c;
}

.agent-header {
  padding: 10px 16px;
  font-size: 11px;
  font-weight: bold;
  letter-spacing: 1px;
  color: #858585;
  text-transform: uppercase;
  border-bottom: 1px solid #3c3c3c;
}

.agent-messages {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.message {
  padding: 8px 10px;
  border-radius: 6px;
  font-size: 13px;
  line-height: 1.4;
}

.message.assistant {
  background-color: #2d2d2d;
  color: #cccccc;
  align-self: flex-start;
}

.message.user {
  background-color: #094771;
  color: #ffffff;
  align-self: flex-end;
}

.agent-input {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 10px;
  border-top: 1px solid #3c3c3c;
}

.agent-input textarea {
  resize: none;
  background-color: #1e1e1e;
  color: #d4d4d4;
  border: 1px solid #3c3c3c;
  border-radius: 4px;
  padding: 6px 8px;
  font: inherit;
  font-size: 13px;
}

.agent-input button {
  align-self: flex-end;
  background-color: #0e639c;
  color: #ffffff;
  border: none;
  border-radius: 4px;
  padding: 6px 14px;
  font: inherit;
  font-size: 13px;
  cursor: pointer;
}

.agent-input button:hover {
  background-color: #1177bb;
}
</style>
