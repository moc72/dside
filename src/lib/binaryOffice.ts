import * as CFB from "cfb";

import { t } from "../i18n";
import { base64ToArrayBuffer, escapeHtml, textOnlyPreview } from "./preview";
import { renderDocx } from "./office";

/**
 * The two binary formats from before Office 2007: .doc and .ppt are OLE containers (the
 * "compound file" format), not zips. The container is read with the SheetJS `cfb` reader;
 * what is inside is read here, by hand, because no maintained browser library does it.
 *
 * Both formats are read as *text*. Anything that depends on layout, images or slide design
 * is out of reach without a real converter, and this says so instead of guessing.
 */
const MAX_CHARACTERS = 400_000;
const MAX_SLIDES = 80;

type Bytes = Uint8Array;

function toBytes(content: unknown): Bytes {
  if (content instanceof Uint8Array) return content;
  if (content instanceof ArrayBuffer) return new Uint8Array(content);
  if (Array.isArray(content)) return Uint8Array.from(content as number[]);
  if (content && typeof content === "object" && "buffer" in (content as ArrayBufferView)) {
    const view = content as ArrayBufferView;
    return new Uint8Array(view.buffer, view.byteOffset, view.byteLength);
  }
  throw new Error("unexpected stream type");
}

function stream(container: CFB.CFB$Container, name: string): Bytes {
  const entry = CFB.find(container, name);
  if (!entry || !entry.content) throw new Error(`stream ${name} is missing`);
  return toBytes(entry.content);
}

function open(base64: string, name: string): Bytes {
  const container = CFB.read(new Uint8Array(base64ToArrayBuffer(base64)), { type: "array" });
  return stream(container, name);
}

// ---------------------------------------------------------------------------
// .doc — Word 97-2003
// ---------------------------------------------------------------------------

/** Piece table of the document: which character ranges sit where in the WordDocument stream. */
interface Piece {
  cpStart: number;
  cpEnd: number;
  offset: number;
  compressed: boolean;
}

export async function renderLegacyWord(base64: string): Promise<string> {
  // A .doc that is really a .docx happens (Word saves both under the same name, and people
  // rename files). The zip magic gives it away, so hand it to the OOXML reader.
  const head = new Uint8Array(base64ToArrayBuffer(base64), 0, 2);
  if (head[0] === 0x50 && head[1] === 0x4b) return renderDocx(base64);

  const word = open(base64, "WordDocument");
  const view = new DataView(word.buffer, word.byteOffset, word.byteLength);

  if (view.getUint16(0, true) !== 0xa5ec) throw new Error("not a Word document");
  const nFib = view.getUint16(2, true);
  if (nFib < 101) throw new Error("Word 6/95 files are not supported");
  if (view.getUint16(0x0a, true) & 0x0004) {
    // fComplex: the text is written in more than one pass. Rare, and reading it wrong would
    // show words that are not there.
    throw new Error("complex Word document");
  }

  const tableName = view.getUint16(0x0a, true) & 0x0200 ? "1Table" : "0Table";
  const ccpText = view.getUint32(0x4c, true);
  const fcClx = view.getUint32(0x1a2, true);
  const lcbClx = view.getUint32(0x1a6, true);

  const table = open(base64, tableName);
  const pieces = readPieceTable(table, fcClx, lcbClx);
  const text = readText(word, pieces, ccpText);

  return textOnlyPreview(cleanWordText(text));
}

function readPieceTable(table: Bytes, fcClx: number, lcbClx: number): Piece[] {
  const view = new DataView(table.buffer, table.byteOffset, table.byteLength);
  const end = Math.min(fcClx + lcbClx, table.byteLength);

  // The Clx is a list of property modifiers followed by the piece table (Pcdt, marker 0x02).
  let at = fcClx;
  while (at < end) {
    const marker = table[at];
    if (marker === 0x01) {
      at += 3 + view.getUint16(at + 1, true);
      continue;
    }
    if (marker !== 0x02) throw new Error("unreadable piece table");

    const length = view.getUint32(at + 1, true);
    const count = Math.floor((length - 4) / 12);
    if (count <= 0) throw new Error("empty piece table");

    const cps = at + 5;
    const pcds = cps + (count + 1) * 4;
    const pieces: Piece[] = [];

    for (let index = 0; index < count; index += 1) {
      const fc = view.getUint32(pcds + index * 8 + 2, true);
      // Bit 30 marks text stored as one byte per character (CP1252) instead of UTF-16.
      const compressed = (fc & 0x4000_0000) !== 0;
      pieces.push({
        cpStart: view.getUint32(cps + index * 4, true),
        cpEnd: view.getUint32(cps + (index + 1) * 4, true),
        offset: compressed ? (fc & 0x3fff_ffff) >>> 1 : fc,
        compressed,
      });
    }
    return pieces;
  }
  throw new Error("no piece table");
}

function readText(word: Bytes, pieces: Piece[], ccpText: number): string {
  const ascii = new TextDecoder("windows-1252");
  const wide = new TextDecoder("utf-16le");
  let text = "";

  for (const piece of pieces) {
    const from = Math.max(piece.cpStart, 0);
    const to = Math.min(piece.cpEnd, ccpText);
    if (to <= from) continue;

    if (piece.compressed) {
      const start = piece.offset + (from - piece.cpStart);
      text += ascii.decode(word.subarray(start, start + (to - from)));
    } else {
      const start = piece.offset + (from - piece.cpStart) * 2;
      text += wide.decode(word.subarray(start, start + (to - from) * 2));
    }
    if (text.length > MAX_CHARACTERS) break;
  }
  return text;
}

/** Word's control characters say where paragraphs, cells and field codes begin and end. */
function cleanWordText(text: string): string {
  return text
    .replace(/\u0013[^\u0014]*\u0014/g, "")
    .replace(/\r/g, "\n")
    .replace(/[\u000b\u000c\u000e]/g, "\n")
    .replace(/\u0007/g, "\t")
    .replace(/[\u0001\u0002\u0003\u0004\u0005\u0008\u0013\u0014\u0015\u0019\u001f]/g, "")
    .replace(/\u001e/g, "-")
    .replace(/[^\S\n\t]+/g, " ")
    .replace(/[ \t]+\n/g, "\n")
    .replace(/\n{3,}/g, "\n\n");
}

// ---------------------------------------------------------------------------
// .ppt — PowerPoint 97-2003
// ---------------------------------------------------------------------------

const RECORD_SLIDE = 0x03ee;
const ATOM_TEXT_CHARS = 0x0fa0;
const ATOM_TEXT_BYTES = 0x0fa8;

export function renderLegacySlides(base64: string): string {
  const document = open(base64, "PowerPoint Document");
  const view = new DataView(document.buffer, document.byteOffset, document.byteLength);
  const slides: string[][] = [];

  // Every record starts with 8 bytes: version/instance, type, length. Version 0x0F marks a
  // container whose payload is more records.
  const walk = (from: number, to: number, insideSlide: boolean): void => {
    let at = from;
    while (at + 8 <= to) {
      const version = view.getUint16(at, true);
      const type = view.getUint16(at + 2, true);
      const length = view.getUint32(at + 4, true);
      const body = at + 8;
      const next = body + length;
      if (next > to) break;

      if ((version & 0x000f) === 0x000f) {
        if (type === RECORD_SLIDE) slides.push([]);
        walk(body, next, insideSlide || type === RECORD_SLIDE);
      } else if (insideSlide && (type === ATOM_TEXT_CHARS || type === ATOM_TEXT_BYTES)) {
        const payload = document.subarray(body, next);
        const text =
          type === ATOM_TEXT_CHARS
            ? new TextDecoder("utf-16le").decode(payload)
            : new TextDecoder("windows-1252").decode(payload);
        for (const line of text.split(/\r|\u000b/)) {
          const trimmed = line.replace(/[\u0000-\u0008\u000e-\u001f]/g, "").trim();
          if (trimmed) slides[slides.length - 1].push(trimmed);
        }
      }
      at = next;
    }
  };

  walk(0, document.byteLength, false);

  const html: string[] = [];
  slides.slice(0, MAX_SLIDES).forEach((lines, index) => {
    if (lines.length === 0) return;
    const heading = escapeHtml(t("viewFile.slide", { n: index + 1 }));
    const body = lines
      .map(
        (line, position) =>
          `<p${position === 0 ? ' class="slide-title"' : ""}>${escapeHtml(line)}</p>`,
      )
      .join("");
    html.push(`<section class="slide"><h3>${heading}</h3>${body}</section>`);
  });

  if (html.length === 0) {
    return `<p class="truncated">${escapeHtml(t("viewFile.noText"))}</p>`;
  }
  return html.join("");
}
