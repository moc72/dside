import DOMPurify from "dompurify";
import { marked } from "marked";

export type PreviewKind = "markdown" | "image" | "pdf" | "text";

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

export function extensionOf(path: string): string {
  const name = path.split("/").pop() ?? path;
  return name.includes(".") ? name.split(".").pop()!.toLowerCase() : "";
}

export function previewKindFor(path: string): PreviewKind {
  const extension = extensionOf(path);
  if (markdownExtensions.has(extension)) return "markdown";
  if (imageExtensions.has(extension)) return "image";
  if (pdfExtensions.has(extension)) return "pdf";
  return "text";
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
