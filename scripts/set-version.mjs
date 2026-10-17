#!/usr/bin/env node
/**
 * One version, three manifests. A release tag is the source of truth:
 *
 *   node scripts/set-version.mjs 0.2.0      (or v0.2.0 — the leading v is dropped)
 *
 * Updates package.json, src-tauri/Cargo.toml and src-tauri/tauri.conf.json. Cargo.lock
 * catches up on the next build. With no argument it only prints what is set now, which is
 * what the release workflow uses to check that a tag and the manifests agree.
 */
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const files = {
  package: join(root, "package.json"),
  cargo: join(root, "src-tauri", "Cargo.toml"),
  tauri: join(root, "src-tauri", "tauri.conf.json"),
};

const read = (path) => readFileSync(path, "utf8");

function currentVersion() {
  return JSON.parse(read(files.package)).version;
}

function setJsonVersion(path, version) {
  const parsed = JSON.parse(read(path));
  if (parsed.version === version) return false;
  parsed.version = version;
  writeFileSync(path, `${JSON.stringify(parsed, null, 2)}\n`);
  return true;
}

/** Only the [package] version, not a dependency's version line. */
function setCargoVersion(version) {
  const source = read(files.cargo);
  const header = source.indexOf("[package]");
  const body = header === -1 ? 0 : header;
  const match = /^version\s*=\s*"([^"]*)"/m.exec(source.slice(body));
  if (!match) throw new Error("no version in src-tauri/Cargo.toml [package]");
  if (match[1] === version) return false;

  const start = body + match.index;
  writeFileSync(
    files.cargo,
    source.slice(0, start) + `version = "${version}"` + source.slice(start + match[0].length),
  );
  return true;
}

const argument = process.argv[2];
if (!argument) {
  console.log(currentVersion());
  process.exit(0);
}

const version = argument.replace(/^v/, "").trim();
if (!/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/.test(version)) {
  console.error(`not a version: ${argument}`);
  process.exit(1);
}

const changed = [
  setJsonVersion(files.package, version) && "package.json",
  setCargoVersion(version) && "Cargo.toml",
  setJsonVersion(files.tauri, version) && "tauri.conf.json",
].filter(Boolean);

console.log(changed.length ? `${version}: updated ${changed.join(", ")}` : `${version}: already set`);
