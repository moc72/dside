import DOMPurify from "dompurify";
import { marked } from "marked";

export type PreviewKind = "markdown" | "image" | "pdf" | "docx" | "sheet" | "rtf" | "unsupported" | "text";

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
const docxExtensions = new Set(["docx", "odt"]);
const sheetExtensions = new Set(["xlsx", "xls", "ods", "csv"]);
const rtfExtensions = new Set(["rtf"]);
/** Old binary Word documents: no library reads these without a converter. */
const unsupportedExtensions = new Set(["doc", "ppt", "pptx", "odp"]);

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
  if (sheetExtensions.has(extension)) return "sheet";
  if (rtfExtensions.has(extension)) return "rtf";
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
export function renderMarkdown(source: string): string {
  const html = marked.parse(source, { async: false, gfm: true }) as string;
  return DOMPurify.sanitize(html, { USE_PROFILES: { html: true } });
}
