# DS·IDE

An agentic desktop IDE on **Tauri v2** (no Electron) with **DeepSeek** as the model — and a
partner inside it, not just a tool. The partner chooses its own name, once; until it does,
it goes by the placeholder **Seeky**. It is called **Ziggy** now.

> The authoritative status, every design decision and the open list live in
> [PROGRESS.md](./PROGRESS.md) — not in this file.

![DS·IDE in Chinese: chat, work area with the README, and the file tree](docs/dside_zh.png)

*The same window in Chinese — one of six languages. The chat runs next to the files; the
work area shows a preview, not an editor.*

## What it does

**The partner**

- Agentic chat with streaming and real tool execution — reading, writing, editing, shell,
  search, memory, sub-agents.
- One answer is a **timeline**: thinking, text and tool calls in the order they happened.
  Tool rows open to show the command, its output, and whether it ran without asking.
- **Long-term memory** the user can read, correct and retire (nothing is deleted), with a
  visible marker for what the user wrote or rewrote.
- **Context ring** showing how full the model's window is, and **compression**: older turns
  are folded into a summary the model writes itself (automatically at 75 %, or on click).
- **Image attachments** by paste or picker; they stay with the message and are sent along.

**Files and formats**

- File tree, source view (Monaco, read-only), and previews without external tools:
  Markdown, PDF, images, Word (docx, doc, odt), Excel (all sheets, values **or formulas**),
  presentations (pptx, ppt, odp), RTF.
- Integrated browser.
- **Skills** live in `~/.ds-ide/skills` and can be written in the app.

**The app**

- Sessions, open tabs, the last file and the window geometry survive a restart.
- Settings → a group named after the partner (memory, name, the user's own instructions).
- **Everyday mode**: no shell, no code panels, no code editor — for documents and the web.
- Six languages (de, en, es, fr, ru, zh-Hans), light and dark theme, adjustable reading size.
- API key in the OS keychain, never in the frontend. Everything local in SQLite.

## Requirements

- Node + pnpm (**pnpm 10** — another major gives `ERR_PNPM_UNEXPECTED_STORE`, the store
  layouts are incompatible)
- Rust (stable)
- Linux: WebKitGTK 4.1, libsoup3, GTK3

## Run it

```sh
pnpm install
pnpm tauri dev
```

First start: enter the API key under **Settings → Usage & Model → API Key**. It is stored in
the OS keychain. Without **"Full permissions"** enabled, `write`, `edit` and `bash` do not
run — and the tool row says so.

## Build an installer

```sh
pnpm tauri build
```

Produces `src-tauri/target/release/bundle/`: `.deb`, `.rpm` and `.AppImage` on Linux
(`dmg` on macOS, `nsis` on Windows). GitHub Actions builds all three platforms on a tag;
see `.github/workflows/release.yml`.

## Test

```sh
pnpm build                                                   # vue-tsc + vite
cd src-tauri && cargo test                                   # unit tests
DS_IDE_API_KEY=... cargo test --lib live_deepseek -- --ignored --nocapture
```

The live tests talk to the real provider and cost tokens; they are ignored by default.

## Structure

```
src/                  Vue frontend (components, stores, i18n)
src/engine/           persona, name pool, mode policy
src/lib/              previews, office readers, persisted UI state
src-tauri/src/        Rust: engine/ (LLM loop + tools), db.rs (SQLite), secret.rs (keyring)
scripts/              small tools (e.g. the name-table importer)
```

## Origins and acknowledgements

Protocol formats and tool names were adopted from **MiniMax Code** (MIT):
`packages/protocol/src/runtime.ts`, `agent-tools/src/desktop/builtin-defs.ts`. The persona
was written independently; no identity-defining text was copied. Typeface: **Ubuntu Font
Licence 1.0** (`public/fonts/UFL.txt`). Both are credited in the app under Settings → About.

The reference checkouts this project was built against (`minimax-code-main`,
`minimax-jarvis`, `kilocode-main`) are **not** part of this repository; where to find them is
recorded in PROGRESS.md §30–§31.

## Licence

MIT — see [LICENSE](./LICENSE). Protocol formats and tool names that came from MiniMax Code
carry their MIT notice too; the bundled Ubuntu font keeps its own licence
(`public/fonts/UFL.txt`). What that means in practice: use it, change it, ship it, even
commercially — just keep the copyright notice.
