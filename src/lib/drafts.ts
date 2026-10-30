import { loadJson, saveJson } from "./persist";

/**
 * What was typed but not sent, per session. The field sits under a long conversation and
 * holds the sentence someone left for later — a restart, a crash, or a switch to another
 * task must not swallow it. Plain text only: a draft is a thought in progress. Attachments
 * keep their own lifecycle and are not resumed behind the user's back.
 */
const KEY = "ds-ide.composer-drafts.v1";
const MAX_SESSIONS = 40;
const MAX_CHARS = 20_000;

interface Stored {
  text: string;
  /** Last write, used only to decide which drafts to drop when there are too many. */
  at: number;
}

type Book = Record<string, Stored>;

function read(): Book {
  const book = loadJson<Book>(KEY, {});
  return book && typeof book === "object" ? book : {};
}

export function draftFor(sessionId: string): string {
  return read()[sessionId]?.text ?? "";
}

export function rememberDraft(sessionId: string, text: string): void {
  const book = read();
  const text_ = text.length > MAX_CHARS ? text.slice(0, MAX_CHARS) : text;
  if (!text_.trim()) {
    if (!book[sessionId]) return;
    delete book[sessionId];
  } else {
    book[sessionId] = { text: text_, at: Date.now() };
  }

  const ids = Object.keys(book);
  if (ids.length > MAX_SESSIONS) {
    ids.sort((a, b) => (book[a]?.at ?? 0) - (book[b]?.at ?? 0));
    for (const id of ids.slice(0, ids.length - MAX_SESSIONS)) delete book[id];
  }
  saveJson(KEY, book);
}

export function forgetDraft(sessionId: string): void {
  const book = read();
  if (!(sessionId in book)) return;
  delete book[sessionId];
  saveJson(KEY, book);
}
