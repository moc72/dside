export interface FileNode {
  name: string;
  path: string;
  isFolder: boolean;
  language?: string;
  content?: string;
  children?: FileNode[];
  expanded?: boolean;
  /** Whether `children` were read; folders load lazily when expanded. */
  loaded?: boolean;
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

/** One snapshot of a file before the partner changed it. */
export interface FileVersionInfo {
  id: number;
  path: string;
  origin: string;
  existed: boolean;
  bytes: number;
  createdAt: number;
}

/** The compressed summary of a session, shown as the "where we were" card. */
export interface SessionResume {
  summary: string;
  throughMessageId: number;
}

export type SessionGroup = "pinned" | "scheduled" | "project" | "team";

export interface Session {
  id: string;
  title: string;
  group: SessionGroup;
  workspace: string;
}

/**
 * One piece of an answer, in the order it happened. A turn is not "some text plus some tools":
 * the model thinks, answers, runs something, answers again, and the reader needs exactly that
 * order. Keeping text and tools in separate fields (and rendering text first) put every later
 * answer above the tools that caused it.
 */
export type MessagePart =
  | { kind: "text"; text: string }
  | { kind: "reasoning"; text: string }
  | { kind: "tool"; tool: ToolActivity }
  /** A picture the user attached. `preview` is the small copy the chat shows. */
  | { kind: "image"; text: string; path: string; name: string; mime: string; preview: string };

export interface ChatMessage {
  id: string;
  role: "user" | "assistant";
  /** Everything said, joined — for copying and for the plain-text view. */
  text: string;
  reasoning?: string;
  streaming?: boolean;
  error?: string;
  tools?: ToolActivity[];
  /** The timeline. Empty for user messages. */
  parts?: MessagePart[];
}

export interface ToolActivity {
  id: string;
  name: string;
  arguments?: string;
  output?: string;
  isError?: boolean;
  durationMs?: number;
  /** A data URL for picture-producing tools (screenshot). */
  image?: string;
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
    image?: string;
  };
  status?: string;
  stopReason?: string;
  action?: {
    id: string;
    kind: string;
    question: string;
    options: string[];
  };
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

export type WorkAreaTabKind =
  | "progress"
  | "file"
  | "skills"
  | "schedules"
  | "terminal"
  | "git"
  | "logs"
  | "changes";

export interface WorkAreaTab {
  id: string;
  kind: WorkAreaTabKind;
  filePath?: string | null;
}

export interface TurnFileDiff {
  path: string;
  before: string;
  after: string;
  existedBefore: boolean;
  existsAfter: boolean;
  origin: string;
}

export interface TurnDiff {
  turnId: string;
  files: TurnFileDiff[];
  canUndo: boolean;
}

export type AppView = "chat" | "settings";

export type AppMode = "coding" | "everyday";

/** How much the companion may do without asking. */
export type PermissionMode = "read" | "ask" | "full";

/** The companion's shell policy; the user's terminal is never restricted by this. */
export type CommandPolicy = "all" | "allowlist";

export type Appearance = "light" | "dark" | "system";
