import DOMPurify from "dompurify";
import mammoth from "mammoth";
import * as XLSX from "xlsx";

import { base64ToArrayBuffer, escapeHtml } from "./preview";

const MAX_ROWS = 300;
const MAX_COLUMNS = 40;

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
      ? `<p class="truncated">… ${rows.length - MAX_ROWS} weitere Zeilen nicht angezeigt</p>`
      : "";

  return `<table>${body}</table>${truncated}`;
}
