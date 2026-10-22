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
cd src-tauri && cargo test --lib  # 34 passed, 13 ignored (live tests need DS_IDE_API_KEY)
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
- **Memory:** long-term memory the user can read, search, correct and retire; notes can be
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
- **Terminal:** a real PTY (portable-pty: ConPTY on Windows, the Unix PTY elsewhere) in the
  work area. The terminal stays alive while its tab exists; closing the tab stops the shell.
  Keys go to the shell while it has focus.
- **Git:** status (porcelain), stage/unstage/stage-all, diff, commit, local branch switching
  and `pull --ff-only` from the work area. No push or merge UI; the terminal is there for that.
- **Guardrails:** the companion's bash tool can be restricted to an editable allowlist of
  program names; chains with unknown programs, redirection, backticks and `$()` are rejected.
  `write`/`edit` can be kept inside the workspace (including no `..` and no symlink escape).
  The default is allowlist + workspace-only. Your own terminal is not restricted. This is a
  policy, not an OS sandbox; allowed interpreters such as node/python remain powerful.
- **Workspace picker:** a folder browser on the existing `read_dir_children` command (path
  field, up, folder list, choose) in the first-start dialog and under Settings → General.
  No native-dialog plugin or new dependency.
- **Eyes:** the composer's eye button captures the virtual desktop and attaches a 2000px
  JPEG. The `screenshot` tool lets the companion take a capture itself: Fragen asks first,
  Mitlesen/Ruder run directly, the picture appears in the tool row and is attached as a user
  image to the next model call. **Limit:** the current DeepSeek models are text-only in this
  setup and say they cannot see the image; real sight needs a vision-capable model. The full
  capture is temporary; Windows uses PowerShell, macOS `screencapture`, Linux one of
  grim/gnome-screenshot/spectacle/scrot/ImageMagick.
- **Ears:** an in-memory log ring (400 lines) receives the app's and engine's log lines. The
  Logs tab shows them; the `logs` tool lets the companion read them. Nothing is persisted.
- **Rudder:** three permission stages — **Mitlesen** (read-only), **Fragen** (approve every
  write/edit/bash), **Ruder** (full access). The composer select and Settings use the same
  three stages; choosing Ruder goes through a warning dialog with a required checkbox. New
  installations start in Fragen; the old full-authorization boolean is migrated. In Fragen
  the engine pauses with the concrete command/content before the tool runs and offers
  once/session/deny; a session grant lives only in memory and is cleared when the stage
  changes. Scheduled runs are read-only in Fragen (nobody can answer). Even in Ruder, a small
  danger floor (`rm -rf /`, `sudo`, `mkfs`, `git reset --hard`, `curl | sh`, …) asks once.
- **Languages:** de, en, es, fr, ru, zh-Hans; light/dark theme and reading size.

## Trust and safety

- `write`, `edit` and `bash` follow the three permission stages. **Mitlesen** denies them,
  **Fragen** pauses the turn and shows the concrete command or change for allow/deny, and
  **Ruder** runs them without a prompt after the consent dialog. New installations start in
  Fragen; the old boolean is migrated.
- There is **no OS sandbox**. The guardrails are a policy: an allowlist of program names and
  a workspace-only write path. Allowed interpreters and the user's own terminal are not
  confined. File history is the undo net, not a security boundary. A small hard floor for
  destructive shell patterns asks even in Ruder. In Fragen, sub-agent writes are still denied
  because the sub-agent cannot reach the question panel; use Ruder for those. The terminal
  and the Git panel are user-initiated tools, not model tools.
- The **Eyes** button captures the whole virtual desktop; the `screenshot` tool does it on
  the companion's request (Fragen asks first). The image reaches the request and the tool
  row, but the current DeepSeek model reports that it cannot see pictures. On macOS, Screen
  Recording permission is required; on Linux one of the capture tools must be installed.
- The API key never reaches the frontend; the database is local. Do not commit
  `secrets.json`; it is gitignored.

## Known limitations

- No signed installers, no notarization, no auto-updater. Windows SmartScreen and macOS
  Gatekeeper warn; the app is still installable.
- The published macOS DMG is **aarch64** (Apple Silicon). Intel needs another CI target
  (`macos-13`) or a universal build.
- The Git panel has no push or merge UI; it has branch switching and pull. The terminal has
  one shell per tab and no profile picker.
- No vision model: screenshots (manual or the `screenshot` tool) reach the provider as image
  attachments, but the current DeepSeek models say they cannot see them. A vision-capable
  model is required for actual sight.
- Full authorization is a per-action approval in Fragen, not a sandbox or capability layer.
  The Rudder dialog is the consent for full access. The allowlist/workspace-only guardrails
  are a policy; a real OS sandbox/per-command capability layer is still open.
- The main frontend chunk is now ~224 KB; Monaco, PDF, Office, terminal, Git and logs are
  lazy chunks.
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
