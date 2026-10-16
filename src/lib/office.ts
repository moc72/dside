import DOMPurify from "dompurify";
import JSZip from "jszip";
import mammoth from "mammoth";
import * as XLSX from "xlsx";

import { base64ToArrayBuffer, escapeHtml } from "./preview";
import { t } from "../i18n";

const MAX_ROWS = 300;
const MAX_COLUMNS = 40;
const MAX_SLIDES = 80;

/** Word: mammoth produces HTML, which then goes through the same sanitiser as markdown. */
export async function renderDocx(base64: string): Promise<string> {
  const result = await mammoth.convertToHtml({ arrayBuffer: base64ToArrayBuffer(base64) });
  return DOMPurify.sanitize(result.value, { USE_PROFILES: { html: true } });
}

/**
 * Spreadsheets: the table is built here, cell by cell, from values we escape ourselves.
 * Letting the library emit HTML would mean trusting it twice.
 */
export function renderSheet(base64: string): string {
  const workbook = XLSX.read(base64ToArrayBuffer(base64), { type: "array" });
  const name = workbook.SheetNames[0];
  if (!name) return "";

  const sheet = workbook.Sheets[name];
  const rows = XLSX.utils.sheet_to_json<unknown[]>(sheet, {
    header: 1,
    blankrows: false,
    defval: "",
  });

  const body = rows
    .slice(0, MAX_ROWS)
    .map(
      (row) =>
        `<tr>${row
          .slice(0, MAX_COLUMNS)
          .map((value) => `<td>${escapeHtml(String(value ?? ""))}</td>`)
          .join("")}</tr>`,
    )
    .join("");

  const truncated =
    rows.length > MAX_ROWS
      ? `<p class="truncated">${escapeHtml(t("viewFile.moreRows", { n: rows.length - MAX_ROWS }))}</p>`
      : "";

  return `<table>${body}</table>${truncated}`;
}

/**
 * Presentations: a pptx is a zip full of slide XML, and no browser library renders its real
 * layout. So this reads what a reader actually needs back — the title and the bullet text of
 * every slide. Text, not a thumbnail, and it says so by looking like a list of slides.
 */
export async function renderSlides(base64: string): Promise<string> {
  const zip = await JSZip.loadAsync(base64ToArrayBuffer(base64));
  const names = Object.keys(zip.files)
    .filter((name) => /^ppt\/slides\/slide\d+\.xml$/.test(name))
    .sort((a, b) => slideNumberOf(a) - slideNumberOf(b));
  if (names.length === 0) return "";

  const parser = new DOMParser();
  const slides: string[] = [];

  for (const name of names.slice(0, MAX_SLIDES)) {
    const file = zip.file(name);
    if (!file) continue;

    const slide = parser.parseFromString(await file.async("string"), "application/xml");
    const paragraphs: string[] = [];
    // Namespace-agnostic on purpose: the `a:` prefix is what every writer emits, but the
    // local names (`p` for a paragraph, `t` for a run of text) are what we mean.
    for (const paragraph of Array.from(slide.getElementsByTagNameNS("*", "p"))) {
      const text = Array.from(paragraph.getElementsByTagNameNS("*", "t"))
        .map((run) => run.textContent ?? "")
        .join("")
        .trim();
      if (text) paragraphs.push(text);
    }
    if (paragraphs.length === 0) continue;

    const body = paragraphs
      .map(
        (text, position) =>
          `<p${position === 0 ? ' class="slide-title"' : ""}>${escapeHtml(text)}</p>`,
      )
      .join("");
    const heading = escapeHtml(t("viewFile.slide", { n: slideNumberOf(name) }));
    slides.push(`<section class="slide"><h3>${heading}</h3>${body}</section>`);
  }

  if (slides.length === 0) {
    return `<p class="truncated">${escapeHtml(t("viewFile.slidesWithoutText"))}</p>`;
  }

  return DOMPurify.sanitize(slides.join(""), { USE_PROFILES: { html: true } });
}

function slideNumberOf(name: string): number {
  const match = /slide(\d+)\.xml$/.exec(name);
  return match ? Number(match[1]) : 0;
}
