<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from "vue";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import type { FileNode } from "../types";

const props = defineProps<{
  files?: FileNode[];
}>();

const host = ref<HTMLDivElement | null>(null);

const PROMPT = "\x1b[38;5;114m➜\x1b[0m \x1b[38;5;39mide\x1b[0m $ ";
const BANNER =
  "\x1b[1mDemo-Terminal\x1b[0m\r\nEchte Shell-Anbindung folgt über ein PTY-Backend in Rust.\r\nBefehle: help, clear, echo, ls, date\r\n";

let term: Terminal | null = null;
let fit: FitAddon | null = null;
let observer: ResizeObserver | null = null;
let line = "";

function run(input: string) {
  if (!term) return;
  const parts = input.trim().split(/\s+/);
  const cmd = parts[0] ?? "";
  const args = parts.slice(1);

  switch (cmd) {
    case "":
      return;
    case "help":
      term.writeln("help  clear  echo <text>  ls  date");
      return;
    case "clear":
      term.clear();
      return;
    case "echo":
      term.writeln(args.join(" "));
      return;
    case "date":
      term.writeln(new Date().toString());
      return;
    case "ls":
      term.writeln((props.files ?? []).map((node) => node.name + (node.isFolder ? "/" : "")).join("   "));
      return;
    default:
      term.writeln(`command not found: ${cmd}`);
  }
}

function newline() {
  term?.write("\r\n" + PROMPT);
}

onMounted(() => {
  if (!host.value) return;

  const instance = new Terminal({
    cursorBlink: true,
    fontSize: 13,
    fontFamily: "Menlo, 'DejaVu Sans Mono', Consolas, monospace",
    theme: { background: "#1e1e1e", foreground: "#d4d4d4", cursor: "#aeafad" },
  });
  const fitAddon = new FitAddon();

  instance.open(host.value);
  instance.loadAddon(fitAddon);
  fitAddon.fit();

  instance.write(BANNER + PROMPT);
  instance.onData((data) => {
    for (const char of data) {
      if (char === "\r") {
        instance.write("\r\n");
        run(line);
        line = "";
        instance.write(PROMPT);
      } else if (char === "\x7f") {
        if (line.length > 0) {
          line = line.slice(0, -1);
          instance.write("\b \b");
        }
      } else if (char === "\x03") {
        instance.write("^C");
        line = "";
        newline();
      } else if (char >= " ") {
        line += char;
        instance.write(char);
      }
    }
  });

  term = instance;
  fit = fitAddon;

  observer = new ResizeObserver(() => fit?.fit());
  observer.observe(host.value);
});

onBeforeUnmount(() => {
  observer?.disconnect();
  observer = null;
  term?.dispose();
  term = null;
  fit = null;
});
</script>

<template>
  <div class="terminal-panel">
    <div class="terminal-header">TERMINAL</div>
    <div ref="host" class="terminal-host"></div>
  </div>
</template>

<style scoped>
.terminal-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  border-top: 1px solid #3c3c3c;
  background-color: #1e1e1e;
}

.terminal-header {
  padding: 4px 12px;
  font-size: 11px;
  font-weight: bold;
  letter-spacing: 1px;
  color: #858585;
  text-transform: uppercase;
}

.terminal-host {
  flex: 1;
  min-height: 0;
  padding: 0 4px 4px;
}
</style>
