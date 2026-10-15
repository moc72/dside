import DOMPurify from "dompurify";
import { marked } from "marked";

export type PreviewKind = "markdown" | "image" | "text";

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

export function extensionOf(path: string): string {
  const name = path.split("/").pop() ?? path;
  return name.includes(".") ? name.split(".").pop()!.toLowerCase() : "";
}

export function previewKindFor(path: string): PreviewKind {
  const extension = extensionOf(path);
  if (markdownExtensions.has(extension)) return "markdown";
  if (imageExtensions.has(extension)) return "image";
  return "text";
}

/**
 * Markdown to HTML, then through DOMPurify. Rendering foreign text as HTML without
 * sanitising it would hand any document in the workspace script access to the app.
 */
export function renderMarkdown(source: string): string {
  const html = marked.parse(source, { async: false, gfm: true }) as string;
  return DOMPurify.sanitize(html, { USE_PROFILES: { html: true } });
}
