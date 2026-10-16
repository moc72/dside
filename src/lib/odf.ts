import JSZip from "jszip";

import { t } from "../i18n";
import { base64ToArrayBuffer, escapeHtml, textOnlyPreview } from "./preview";

/**
 * OpenDocument (.odt, .odp) is a zip with one `content.xml` in it. There is no renderer here,
 * so this reads the text: the paragraphs of a document, or the text of each page of a
 * presentation. Formatting is dropped on purpose — a preview that pretends to know the
 * layout would be worse than one that says it does not.
 */
const TEXT_NS = "urn:oasis:names:tc:opendocument:xmlns:text:1.0";

async function contentXml(base64: string): Promise<Document> {
  const archive = await JSZip.loadAsync(base64ToArrayBuffer(base64));
  const entry = archive.file("content.xml");
  if (!entry) throw new Error("no content.xml in this file");
  const source = await entry.async("string");
  return new DOMParser().parseFromString(source, "application/xml");
}

/**
 * ODF keeps runs of spaces in `text:s` elements instead of the text itself, and says how many
 * they stand for. Reading only text nodes would silently squeeze them away.
 */
function blockText(block: Element): string {
  let text = "";
  for (const node of Array.from(block.childNodes)) {
    if (node.nodeType === Node.TEXT_NODE) {
      text += node.nodeValue ?? "";
      continue;
    }
    if (node.nodeType !== Node.ELEMENT_NODE) continue;

    const element = node as Element;
    if (element.localName === "s") {
      const count = Number(element.getAttributeNS(TEXT_NS, "c"));
      text += " ".repeat(Number.isFinite(count) && count > 0 ? count : 1);
    } else if (element.localName === "tab") {
      text += "\t";
    } else if (element.localName === "line-break") {
      text += "\n";
    } else if (element.localName === "span") {
      text += blockText(element);
    }
  }
  return text;
}

/** Paragraphs and headings of one part, in document order. */
function blocksOf(root: Element): Element[] {
  const blocks = [
    ...Array.from(root.getElementsByTagNameNS("*", "p")),
    ...Array.from(root.getElementsByTagNameNS("*", "h")),
  ];
  return blocks.sort((a, b) => {
    const relation = a.compareDocumentPosition(b);
    return relation & Node.DOCUMENT_POSITION_FOLLOWING ? -1 : 1;
  });
}

export async function renderOdfText(base64: string): Promise<string> {
  const document = await contentXml(base64);
  return textOnlyPreview(plainText(document.documentElement));
}

/** One card per `draw:page`, numbered by position — that is the order a reader expects. */
export async function renderOdfSlides(base64: string): Promise<string> {
  const document = await contentXml(base64);
  const pages = Array.from(document.getElementsByTagNameNS("*", "page"));
  const slides: string[] = [];

  pages.slice(0, 80).forEach((page, index) => {
    const blocks = blocksOf(page)
      .map((block) => blockText(block).trim())
      .filter((line) => line.length > 0);
    if (blocks.length === 0) return;

    const heading = escapeHtml(t("viewFile.slide", { n: index + 1 }));
    const body = blocks
      .map(
        (line, position) =>
          `<p${position === 0 ? ' class="slide-title"' : ""}>${escapeHtml(line)}</p>`,
      )
      .join("");
    slides.push(`<section class="slide"><h3>${heading}</h3>${body}</section>`);
  });

  if (slides.length === 0) {
    return `<p class="truncated">${escapeHtml(t("viewFile.noText"))}</p>`;
  }
  return slides.join("");
}

function plainText(root: Element): string {
  return blocksOf(root)
    .map((block) => blockText(block).trim())
    .filter((line) => line.length > 0)
    .join("\n");
}
