#!/usr/bin/env node
// Dev client for the DS·IDE UI test bridge (see HANDOVER.md). eval/report work anywhere;
// shot is Linux-only and uses xdotool plus ImageMagick.
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";

const base = (process.env.DSIDE_BRIDGE ?? "http://127.0.0.1:9333").replace(/\/$/, "");
const timeout = Number(process.env.DSIDE_REPORT_TIMEOUT ?? 9000);
const sleep = (ms) => new Promise((done) => setTimeout(done, ms));

async function report() {
  const response = await fetch(`${base}/report`);
  if (!response.ok) throw new Error(`report failed: ${response.status}`);
  return response.json();
}

async function waitForReport(previous) {
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) {
    const current = await report();
    if (current && current.seq > previous) return current;
    await sleep(100);
  }
  return null;
}

async function evaluate(code) {
  const before = await report().catch(() => ({ seq: 0 }));
  const response = await fetch(`${base}/eval`, { method: "POST", body: code });
  if (!response.ok) throw new Error(`eval failed: ${response.status} ${await response.text()}`);
  return waitForReport(before?.seq ?? 0);
}

function screenshot(path) {
  const title = process.env.DSIDE_WINDOW ?? "^DS·IDE$";
  const found = execFileSync("xdotool", ["search", "--name", title], { encoding: "utf8" }).trim();
  const id = found.split("\n").filter(Boolean).at(-1);
  if (!id) throw new Error(`no window matching ${title}`);
  execFileSync("magick", ["import", "-window", id, path]);
  return path;
}

const [command, ...rest] = process.argv.slice(2);
try {
  if (command === "report") {
    console.log(JSON.stringify(await report(), null, 2));
  } else if (command === "eval") {
    const code = rest.length ? rest.join(" ") : readFileSync(0, "utf8");
    const result = await evaluate(code);
    if (!result) {
      console.error("no report arrived within the timeout");
      process.exitCode = 2;
    } else {
      console.log(JSON.stringify(result.payload, null, 2));
    }
  } else if (command === "shot") {
    console.log(screenshot(rest[0] ?? "/tmp/dside-ui.png"));
  } else if (command === "wait") {
    await sleep(Number(rest[0] ?? 1000));
    console.log("ok");
  } else {
    console.error("usage: node scripts/ui-test.mjs report|eval <code>|shot [path]|wait <ms>");
    process.exitCode = 1;
  }
} catch (error) {
  console.error(String(error?.message ?? error));
  process.exitCode = 1;
}
