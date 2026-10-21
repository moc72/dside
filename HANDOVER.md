# DS·IDE — Handover

This is the short handover for taking the project over. `PROGRESS.md` is the authoritative
log (German, one section per round); `AGENTS.md` describes how to work in the repository.

**Status:** the `v0.4.9` release is public. The source tree here is the prepared `v0.4.10`
state (onboarding, diagnostics, file history, project rules); tag it per the release section.

## What it is

A local-first agentic desktop IDE on **Tauri v2** (no Electron):

- **Frontend:** Vue 3 + Vite. Chat timeline, work area (progress, file viewer, skills,
  schedules), Monaco (read-only viewer and diff), integrated previews.
- **Backend:** Rust. DeepSeek streaming loop and tools, SQLite, keyring, skills, schedules,
  native browser windows.
- **Model:** DeepSeek via an OpenAI-compatible chat-completions API. The API key never
  reaches the frontend; Rust reads it from the OS keyring per turn.

## Build and run

Requirements:

- Node 22+ and **pnpm 10** (other majors have incompatible store layouts; pnpm 12 also fails
  on `ERR_PNPM_IGNORED_BUILDS` for `he-tree-vue`).
- Rust stable.
- Linux: WebKitGTK 4.1, libsoup3, GTK3. Windows: WebView2. macOS: the system WebView.

```sh
pnpm install --frozen-lockfile
pnpm tauri dev                    # Vite on 1420 + debug window; cargo must be on PATH
```

Checks and release:

```sh
pnpm build                        # vue-tsc + Vite
cd src-tauri && cargo test --lib  # 27 passed, 12 ignored (live tests need DS_IDE_API_KEY)
pnpm tauri build                  # deb/rpm/AppImage, NSIS, dmg
```

On Windows, a direct `cargo build --release` needs
`--features tauri/custom-protocol`; without it the binary loads `http://localhost:1420`
instead of the embedded frontend. The Tauri CLI sets it itself.

CI: `.github/workflows/release.yml` runs verify (build + Rust tests) and builds Linux,
macOS and Windows on a version tag, creating a **draft** release. The tag is the version;
`scripts/set-version.mjs` keeps the manifests aligned. Signing and notarization are **not**
configured (see limitations).

## First start

A fresh install has an empty `settings` table and shows the onboarding dialog:

- **Mode:** coding (shell, code review, sub-agents, code panels) or everyday (documents and
  web; no shell/code editor).
- **Workspace:** validated with `read_workspace_tree`; the partner reads and writes there.
- **API key:** stored in the OS keyring. The dialog shows whether a key is already present.

Existing installations never see the dialog (the `onboarded` flag is stored with the other
settings). The key can always be changed under **Settings → Usage & model**; **Settings →
Diagnostics** shows the keyring backend, database, workspace, editors and runs a provider
connection test.

## Where state lives

- Tauri app data dir (`com.deepvibe.ds-ide`): `ds-ide.sqlite3`, `attachments/`, the window
  state, and the `secrets.json` fallback (0600) if the keyring is unavailable.
- SQLite: sessions, messages and their timeline, tool calls, todos, memories, settings,
  schedules, session summaries, and `file_versions` (snapshots taken before successful
  `write`/`edit`).
- API key: OS keyring (`ds-ide.deepvibe` / `deepseek-api-key`). On Windows the Credential
  Manager, on macOS the Keychain, on Linux Secret Service. The platform features are
  required in `Cargo.toml`; without them `keyring` silently uses its mock store.
- Skills: `~/.ds-ide/skills/<name>/SKILL.md`.
- Project rules: `AGENTS.md` or `.dside/rules.md` in the workspace; loaded into the system
  prompt for the partner and its sub-agents.

## Features worth knowing

- **Timeline:** thinking, text and tool calls in the order they happened; tool rows open to
  show the command, output and approval state.
- **File history:** every successful `write`/`edit` first stores the old content; the file
  tab's **History** shows the versions, a read-only Monaco diff and **Restore** (the current
  state is backed up before restoring; a snapshot from before the file existed removes it).
- **Retry:** a failed turn shows the cause and a Retry button. The failed tail is rolled back,
  the same user message is sent again (no duplicate), and the error bubble survives a visit
  to Settings. The provider base URL is editable under **Settings → Account & model**.
- **Follow mode:** files the partner reads/writes open in the work area, `web_fetch` opens
  the page in the integrated browser; the eye in the tab bar turns it off.
- **Memory:** long-term memory the user can read, correct and retire; notes can be
  project-scoped (recalled only inside that workspace); context compression folds older
  turns into a summary the model writes.
- **Where we were:** a compressed conversation shows its summary as a sticky card above
  the timeline; the user can open it to re-enter the work.
- **Previews:** Markdown, PDF, images, Word, Excel (values or formulas), presentations, RTF
  without external tools.
- **Editor picker:** opens the workspace in an installed external editor; on Windows the
  `code` shim is launched through `code.cmd` without a console flash.
- **Skills and schedules:** skills on disk; an interval scheduler that runs only while the
  app is open.
- **Languages:** de, en, es, fr, ru, zh-Hans; light/dark theme and reading size.

## Trust and safety

- `write`, `edit` and `bash` require **Full Authorization**; the toggle is in the composer.
  The default for a fresh install is currently **on** — consider changing that default or
  asking at first start before distributing the app broadly.
- There is **no sandbox** and no per-command allowlist yet; the partner runs with the user's
  privileges. `ask_user` exists for questions. File history is the undo net, not a security
  boundary.
- The API key never reaches the frontend; the database is local. Do not commit
  `secrets.json`; it is gitignored.

## Known limitations

- No integrated terminal and no Git panel yet (`TerminalPanel.vue` and the xterm deps are in
  the repo; the PTY backend and the Git UI are not).
- No signed installers, no notarization, no auto-updater. Windows SmartScreen and macOS
  Gatekeeper warn; the app is still installable.
- The published macOS DMG is **aarch64** (Apple Silicon). Intel needs another CI target
  (`macos-13`) or a universal build.
- No memory search; no screenshot tool for the model; no workspace picker after onboarding.
- The main frontend chunk is large (Monaco + PDF + Office); code-splitting is a known task.
- No frontend E2E suite yet; CI runs the build and the Rust unit tests.

## Testing

```sh
pnpm build
cd src-tauri && cargo test --lib
DS_IDE_API_KEY=... cargo test --lib live_deepseek -- --ignored --nocapture
```

The live tests cost tokens and are ignored by default. A fresh-install smoke test was run on
Windows with an emptied `settings` table: the onboarding dialog appeared, coding mode was
chosen, the keyring key was recognized, the diagnostics connection test returned
`2 models · 27.58 USD`, and the original database was restored afterwards.

## Origins and licence

MIT (see `LICENSE`). Protocol formats and tool names were adopted from **MiniMax Code**
(MIT): `packages/protocol/src/runtime.ts`, `agent-tools/src/desktop/builtin-defs.ts`. The
bundled Ubuntu font is under the Ubuntu Font Licence 1.0 (`public/fonts/UFL.txt`). The
reference checkouts used during development are not part of the repository.

## First checks for a new maintainer

1. Build with pnpm 10 and run the Rust tests.
2. Start from an empty app data dir: onboarding, keyring, workspace, provider.
3. Open Settings → Diagnostics and run the connection test.
4. Ask the partner to change a file; open the file tab's History, inspect the diff, restore.
5. Read `PROGRESS.md` §44–§49 for the recent fixes, then `AGENTS.md` for the working rules.
