# DS.IDE

Eine agentische Desktop-IDE auf **Tauri v2** (kein Electron) mit **DeepSeek** als Modell.
Der Partner in der App heißt **Seeky**.

## Stand

Lauffähiges Skeleton: Agent-Chat mit Streaming und echter Tool-Ausführung, Datei-Baum mit
Monaco (nur lesend), Fortschritts- und Gedächtnis-Panel, Einstellungen mit
OS-Schlüsselbund für den API-Key, Persistenz in SQLite.

**Maßgeblicher Stand, Entscheidungen und die offene Liste stehen in
[PROGRESS.md](./PROGRESS.md)** — nicht in dieser Datei.

## Voraussetzungen

- Node + pnpm (**pnpm 10**). Mit einer anderen pnpm-Major gibt es
  `ERR_PNPM_UNEXPECTED_STORE`, weil die Store-Layouts inkompatibel sind.
- Rust (stable)
- Linux: WebKitGTK 4.1, libsoup3, GTK3

## Starten

```sh
pnpm install
pnpm tauri dev
```

Beim ersten Start: **Einstellungen → Nutzung & Modell → API-Key** eintragen. Der Key
landet im OS-Schlüsselbund, nicht im Frontend. Ohne **„Volle Berechtigung"** verweigern
`write`, `edit` und `bash`.

## Prüfen

```sh
pnpm build                                                   # vue-tsc + vite
cd src-tauri && cargo test                                   # Unit-Tests
DS_IDE_API_KEY=... cargo test --lib live_deepseek -- --ignored --nocapture
```

## Struktur

```
src/                  Vue-Frontend (Komponenten, Stores, i18n de/en)
src-tauri/src/        Rust: engine/ (LLM-Loop + Tools), db.rs (SQLite), secret.rs (Keyring)
```

`kilocode-main/`, `minimax-code-main/`, `minimax-jarvis/` liegen nur lokal als Referenz
und sind **nicht** Teil dieses Repos (siehe `.gitignore`).

## Herkunft

Übernommen wurden **Protokollformen und Tool-Namen** aus MiniMax Code (MIT):
`packages/protocol/src/runtime.ts` und `agent-tools/src/desktop/builtin-defs.ts`.
Die Persona ist eigenständig geschrieben; Identitätstexte wurden **nicht** kopiert.
Wird später Material aus jenen Quellen mitgeliefert, muss deren MIT-Attribution
(`NOTICE`/`LICENSE`) mitgeführt werden.
