import DOMPurify from "dompurify";
import JSZip from "jszip";
import mammoth from "mammoth";
import * as XLSX from "xlsx";

import { base64ToArrayBuffer, escapeHtml } from "./preview";
import { t } from "../i18n";

const MAX_COLUMNS = 40;
/** A ceiling for the whole workbook, shared out over its sheets. */
const MAX_CELLS = 12_000;
const MAX_SHEETS = 24;
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
/**
 * Spreadsheets: every sheet, one after the other, each with its own name. Only the first one
 * would be a silent lie about what the file contains — the second sheet is usually where the
 * work is. The number of rows per sheet shrinks as the number of sheets grows, so a workbook
 * with forty sheets cannot flood the viewer.
 */
export function renderSheet(base64: string): string {
  const workbook = XLSX.read(base64ToArrayBuffer(base64), { type: "array" });
  const names = workbook.SheetNames.slice(0, MAX_SHEETS);
  if (names.length === 0) return "";

  const rowsPerSheet = Math.max(20, Math.floor(MAX_CELLS / (names.length * MAX_COLUMNS)));
  const sections: string[] = [];

  for (const name of names) {
    const sheet = workbook.Sheets[name];
    if (!sheet) continue;

    const rows = XLSX.utils.sheet_to_json<unknown[]>(sheet, {
      header: 1,
      blankrows: false,
      defval: "",
    });
    if (rows.length === 0) continue;

    const body = rows
      .slice(0, rowsPerSheet)
      .map(
        (row) =>
          `<tr>${row
            .slice(0, MAX_COLUMNS)
            .map((value) => `<td>${escapeHtml(String(value ?? ""))}</td>`)
            .join("")}</tr>`,
      )
      .join("");

    const truncated =
      rows.length > rowsPerSheet
        ? `<p class="truncated">${escapeHtml(
            t("viewFile.moreRows", { n: rows.length - rowsPerSheet }),
          )}</p>`
        : "";
    const heading =
      names.length > 1 ? `<h3 class="sheet-name">${escapeHtml(name)}</h3>` : "";

    sections.push(`${heading}<table>${body}</table>${truncated}`);
  }

  if (workbook.SheetNames.length > names.length) {
    const rest = workbook.SheetNames.length - names.length;
    sections.push(
      `<p class="truncated">${escapeHtml(t("viewFile.moreSheets", { n: rest }))}</p>`,
    );
  }

  return sections.join("");
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
    return `<p class="truncated">${escapeHtml(t("viewFile.noText"))}</p>`;
  }

  return DOMPurify.sanitize(slides.join(""), { USE_PROFILES: { html: true } });
}

function slideNumberOf(name: string): number {
  const match = /slide(\d+)\.xml$/.exec(name);
  return match ? Number(match[1]) : 0;
}
