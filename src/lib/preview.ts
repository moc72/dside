import DOMPurify from "dompurify";
import { marked } from "marked";

import { t } from "../i18n";

export type PreviewKind =
  | "markdown"
  | "image"
  | "pdf"
  | "text"
  | "rtf"
  | "sheet"
  | "docx"
  | "doc"
  | "odt"
  | "slides"
  | "odp"
  | "ppt"
  | "unsupported";

const markdownExtensions = new Set(["md", "markdown", "mdown", "mkd"]);
const imageExtensions = new Set([
  "png",
  "jpg",
  "jpeg",
  "gif",
  "webp",
  "bmp",
  "svg",
  "avif",
  "ico",
]);
const pdfExtensions = new Set(["pdf"]);
/** Word: the OOXML one goes through mammoth, the old binary one and OpenDocument are read here. */
const docxExtensions = new Set(["docx"]);
const legacyWordExtensions = new Set(["doc"]);
const odfTextExtensions = new Set(["odt"]);
const sheetExtensions = new Set(["xlsx", "xls", "ods", "csv"]);
const rtfExtensions = new Set(["rtf"]);
/** Presentations: pptx is a zip of XML, ppt an OLE container, odp comes from OpenDocument. */
const slideExtensions = new Set(["pptx"]);
const odfSlideExtensions = new Set(["odp"]);
const legacySlideExtensions = new Set(["ppt"]);
/**
 * Containers and binaries nobody can show as a page. Without this list they would land in the
 * text reader and produce a confusing "could not read the file" instead of a clear answer.
 */
const unsupportedExtensions = new Set([
  "zip",
  "7z",
  "rar",
  "tar",
  "gz",
  "bz2",
  "xz",
  "exe",
  "dll",
  "so",
  "dylib",
  "bin",
  "iso",
  "dmg",
  "apk",
  "db",
  "sqlite",
  "sqlite3",
  "mp3",
  "flac",
  "ogg",
  "wav",
  "mp4",
  "mkv",
  "mov",
  "avi",
  "webm",
]);

export function extensionOf(path: string): string {
  const name = path.split("/").pop() ?? path;
  return name.includes(".") ? name.split(".").pop()!.toLowerCase() : "";
}

export function previewKindFor(path: string): PreviewKind {
  const extension = extensionOf(path);
  if (markdownExtensions.has(extension)) return "markdown";
  if (imageExtensions.has(extension)) return "image";
  if (pdfExtensions.has(extension)) return "pdf";
  if (docxExtensions.has(extension)) return "docx";
  if (legacyWordExtensions.has(extension)) return "doc";
  if (odfTextExtensions.has(extension)) return "odt";
  if (sheetExtensions.has(extension)) return "sheet";
  if (rtfExtensions.has(extension)) return "rtf";
  if (slideExtensions.has(extension)) return "slides";
  if (odfSlideExtensions.has(extension)) return "odp";
  if (legacySlideExtensions.has(extension)) return "ppt";
  if (unsupportedExtensions.has(extension)) return "unsupported";
  return "text";
}

export function base64ToArrayBuffer(value: string): ArrayBuffer {
  const binary = atob(value);
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index);
  }
  return bytes.buffer;
}

export function escapeHtml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

/**
 * Crude RTF to text: drop escapes, control words and braces. Enough to read a document,
 * not enough to preserve its look — and honest about which of the two it is.
 */
export function rtfToText(source: string): string {
  return source
    .replace(/\\'[0-9a-f]{2}/gi, "")
    .replace(/\\[a-z]+-?\d* ?/gi, "")
    .replace(/[{}]/g, "")
    .trim();
}

const mimeTypes: Record<string, string> = {
  png: "image/png",
  jpg: "image/jpeg",
  jpeg: "image/jpeg",
  gif: "image/gif",
  webp: "image/webp",
  bmp: "image/bmp",
  svg: "image/svg+xml",
  avif: "image/avif",
  ico: "image/x-icon",
};

export function mimeFor(path: string): string {
  return mimeTypes[extensionOf(path)] ?? "application/octet-stream";
}

/**
 * Markdown to HTML, then through DOMPurify. Rendering foreign text as HTML without
 * sanitising it would hand any document in the workspace script access to the app.
 */
export function renderMarkdown(source: string, breaks = false): string {
  const html = marked.parse(source, { async: false, gfm: true, breaks }) as string;
  return DOMPurify.sanitize(html, { USE_PROFILES: { html: true } });
}

const MAX_PREVIEW_PARAGRAPHS = 4000;

/**
 * The shared look of every "we could read the text, not the layout" preview: plain escaped
 * paragraphs, one line of honesty at the top, and a count when the rest was left out. Used
 * by the binary Word reader and the OpenDocument reader.
 */
export function textOnlyPreview(text: string): string {
  const lines = text
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.length > 0);

  const shown = lines.slice(0, MAX_PREVIEW_PARAGRAPHS);
  const rest = lines.length - shown.length;

  const note = escapeHtml(t("viewFile.textOnly"));
  const more =
    rest > 0 ? `<p class="truncated">${escapeHtml(t("viewFile.moreLines", { n: rest }))}</p>` : "";

  const body = shown.map((line) => `<p>${escapeHtml(line)}</p>`).join("");
  if (!body) {
    return `<p class="truncated">${escapeHtml(t("viewFile.noText"))}</p>`;
  }
  return `<p class="truncated">${note}</p>${body}${more}`;
}
