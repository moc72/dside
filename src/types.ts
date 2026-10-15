export interface FileNode {
  name: string;
  path: string;
  isFolder: boolean;
  language?: string;
  content?: string;
  children?: FileNode[];
  expanded?: boolean;
}

export interface EditorCandidate {
  id: string;
  label: string;
  family: string;
  exec: string;
  args: string[];
  source: string;
  icon?: string | null;
}

export type SessionGroup = "pinned" | "scheduled" | "project" | "team";

export interface Session {
  id: string;
  title: string;
  group: SessionGroup;
  workspace: string;
}

export interface ChatMessage {
  id: string;
  role: "user" | "assistant";
  text: string;
  reasoning?: string;
  streaming?: boolean;
  error?: string;
  tools?: ToolActivity[];
}

export interface ToolActivity {
  id: string;
  name: string;
  arguments?: string;
  output?: string;
  isError?: boolean;
  durationMs?: number;
  running?: boolean;
}

export interface EngineEvent {
  sessionId: string;
  seq: number;
  kind: string;
  text?: string;
  tool?: {
    id: string;
    name: string;
    arguments?: string;
    output?: string;
    isError?: boolean;
    durationMs?: number;
  };
  status?: string;
  stopReason?: string;
  usage?: {
    promptTokens: number;
    completionTokens: number;
    totalTokens: number;
  };
  error?: string;
}

export interface Deliverable {
  id: string;
  group: string;
  label: string;
  done?: boolean;
}

export type WorkAreaTabKind = "progress" | "file" | "browser" | "memory";

export interface WorkAreaTab {
  id: string;
  kind: WorkAreaTabKind;
  filePath?: string | null;
}

export type AppView = "chat" | "settings";

export type AppMode = "coding" | "everyday";

export type Appearance = "light" | "dark" | "system";
