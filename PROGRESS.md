# DS.IDE — Bauplan & Status

> Diese Datei ist der **Restart-Anker nach Kontextkompression**. Sie ist die Wahrheit,
> nicht der Chatverlauf. Nach jeder abgeschlossenen Phase aktualisieren.
> Erstellt: 2026-10-15

## 1. Auftrag

Eine eigene **agentic IDE** mit Tauri (kein Electron) nach Vorlage von **MiniMax Code**.
Erlaubnis vom Nutzer erteilt (2026-10-15, 07:55). Name: **DS.IDE** / "DeepSeek Code".
Arbeitsverzeichnis: `/home/me/Projects/TEST`

Nutzervorgaben (verbindlich):
- **Kein Electron** — Tauri v2.
- **i18n ab dem ersten String** — kein hartkodierter UI-Text. Mindestens `de` + `en`.
- **Kein eingebauter Editor zum Editieren.** Der Code-Editor ist **extern**
  (VS Code / Code OSS / VSCodium / Zed / Cursor / Xcode), per Dropdown gewählt.
  → Wir bieten an: **Systemscan** nach installierten Editoren, manuellen Override,
  Download-Fallback, falls nichts installiert ist.
- Monaco wird **nur als read-only Viewer** im "View File"-Tab genutzt (kein Ersatz für die externe IDE).

## 2. Vorlage: Struktur aus den MiniMax-Code-Screenshots

```
┌──────────────────────────────────────────────────────────────────────┐
│ TitleBar: Projektname (Mitte)            [Editor▾] [🌐] [📁] [▣]     │
├───────────────┬──────────────────────────────┬───────────────────────┤
│ LeftRail      │ AgentConversation (Mitte)    │ WorkArea (rechts)     │
│ - New task    │  - MessageList               │  Tabs:                │
│ - Search      │  - Composer                  │   • Progress/Deliv.   │
│ - Plugins     │    (/Commands,               │   • View File         │
│ - Scheduled   │     Full Authorization,      │   • Browser           │
│ - Site        │     Model▾, Send)            │  (┗ ▣ togglet Panel)  │
│ - Connect Mob.│                              │                       │
│ - Remote ctrl │                              │                       │
│ ─ Pinned      │                              │                       │
│ ─ Scheduled   │                              │                       │
│ ─ Projects    │                              │                       │
│ ─ Agent Team  │                              │                       │
│ [Account][⚙]  │                              │                       │
└───────────────┴──────────────────────────────┴───────────────────────┘
```

Wichtige Lesarten (aus Screenshots bestätigt):
- **Agent-Chat ist die Mitte**, nicht eine Seitenleiste. Editor ist NICHT zentral.
- **Work Area** ist ein Tab-Panel rechts. Der **Datei-Baum lebt IM "View File"-Tab**
  (rechts, ein-/ausklappbar "Hide file tree") — NICHT als linke Spalte.
- Rechts oben drei Aktions-Icons: 🌐 = Browser-Tab öffnen, 📁 = View-File-Tab öffnen,
  ▣ = Work-Area-Panel ein-/ausblenden. Davor das Editor-Dropdown.
- "Progress/Deliverables" ist der Default-Tab der Work Area, gruppiert je Artefakt-Set
  (z.B. "Manuskript", "Mavis markdowns", "Skript: …").
- **Settings** ist eine eigene Seite (nicht Dialog): linke Settings-Nav + rechter Inhalt.
  Gruppen: Preferences (General, Keyboard shortcuts, Personalization),
  Management (Usage & model, Connection, Account), Coding (Code review, Worktree),
  Archived (Archived tasks).
  General-Inhalt: **Mode**-Karten ("For coding" / "For everyday work"), Appearance
  (Light/Dark/System), Show in Menu Bar, Launch on Startup, Desktop Notifications,
  Join Early Access.

## 3. Technische Entscheidungen (fixiert)

| Thema | Entscheidung |
|---|---|
| Shell | Tauri v2 + Vue 3 + TS + Vite 8 + pnpm 10 (bereits gescaffoldet) |
| Paketmanager | **Immer das Shell-pnpm (10.30.3)**. NICHT `/usr/bin/pnpm` (11.28.5) → sonst `ERR_PNPM_UNEXPECTED_STORE`. |
| i18n | Eigener **typisierter Katalog** (`t()` + `Leaves<Messages>`), kein vue-i18n. `de` ist Form-Referenz, `en: Messages` erzwingt gleiche Keys. Persistenz via localStorage. |
| Theme | CSS-Variablen auf `:root` + `[data-theme="light"]`, gesetzt via `document.documentElement.dataset.theme`. |
| Stores | Modul-Singletons mit `reactive`/`ref` + localStorage-Persistenz. Kein Pinia (Overkill). |
| Rust-Befehle | `detect_editors`, `open_workspace_in_editor(editor_id, workspace)`, `read_workspace_tree(root, depth)`, `read_text_file(path)`. |
| Sicherheit | Frontend sendet **niemals** eine Executable. Launch nur über **editor_id**, Rust löst die Binary aus eigener Detection auf. |
| Agent-Backend | **NICHT implementiert.** Konversation ist lokale Demo mit klarem "not wired"-Hinweis. Keine LLM-Calls. |
| Provider | Dropdown listet u.a. DeepSeek (Ziel) + MiniMax etc. Nur Auswahl, keine Anbindung. |
| Monaco | Read-only Viewer im View-File-Tab. External editor bleibt der Ort zum Editieren. |
| Terminal | `TerminalPanel.vue` bleibt im Repo, wird aber **nicht** gemountet (MiniMax hat keins). |

## 4. Datei-Landkarte (Ziel)

```
src/
  main.ts
  App.vue                     Komposition + Ansicht (chat|settings)
  i18n/
    index.ts                  t(), locale, setLocale, Leaves<Messages>
    de.ts                     Form-Referenz
    en.ts                     en: Messages
  types.ts                    FileNode, EditorCandidate, Session, ChatMessage, Deliverable, WorkAreaTab
  lib/
    language.ts               (bestehend) Endung -> Monaco-Sprache
    persist.ts                localStorage-Helfer
  stores/
    settings.ts               mode, appearance, toggles, pinnedEditorId, workspace
    sessions.ts               Sessions, Messages, Deliverables (Beispieldaten)
    workArea.ts               Tabs, aktiver Tab, Sichtbarkeit
  components/
    TitleBar.vue
    LeftRail.vue
    agent/AgentPanel.vue  agent/MessageList.vue  agent/Composer.vue
    workarea/WorkArea.vue  workarea/ProgressTab.vue
    workarea/ViewFileTab.vue  workarea/BrowserTab.vue
    settings/SettingsPage.vue
    FileTree.vue              (bestehend, an reale Daten angepasst)
    MonacoEditor.vue          (bestehend, + readOnly, Model pro Pfad)
    TerminalPanel.vue         (bestehend, ungenutzt)
  data/sampleProject.ts       (bestehend, nur noch Fallback)
src-tauri/
  src/lib.rs                  + Rust-Kommandos
  tauri.conf.json             productName DS.IDE
```

## 5. Status — STAND 2026-10-15, ~10:05 (alles unten ist erledigt)

- [x] Phase 0: PROGRESS.md, App-Identität (`productName`/Titel `DS.IDE`,
      Identifier `com.deepvibe.ds-ide`, Package `ds-ide`)
- [x] Phase 1: i18n (de/en, typisiert), Theme (CSS-Variablen, dark/light/system),
      Stores (settings/sessions/workArea/ui), Types
- [x] Phase 2: Rust — `detect_editors`, `open_workspace_in_editor`,
      `read_workspace_tree`, `read_text_file`, `default_workspace`
- [x] Phase 3: UI — TitleBar, ToolbarActions, LeftRail,
      AgentPanel (MessageList/Composer), WorkArea (Progress/ViewFile/Browser),
      SettingsPage, App
- [x] Phase 4: Build + Verifikation

### Verifikation (Evidenz)
- `pnpm build` (vue-tsc + vite) grün.
- `cargo check` grün; `cargo test detect_editors_smoke` grün.
- Editor-Erkennung real auf dieser Maschine: `code-oss /usr/bin/code-oss`,
  `codium /usr/bin/codium`, `cursor …/cursor.AppImage`, `zed zeditor`
  — 4 Einträge, dedupliziert (12 Rohtreffer → 4).
- `pnpm tauri dev`: Fenster startet, alle Modul-Requests HTTP 200,
  Webview optimiert `@tauri-apps/api/core` + `plugin-opener`.
- Das Flatpak-VSCodium ist inzwischen **nicht mehr installiert**
  (`flatpak list --app` zeigt es nicht mehr) → Detection ist korrekt.

### Bewusst NICHT implementiert (Stubs)
- **Kein Agent-Backend.** Der Composer antwortet mit `message.notWired`; keine LLM-Calls.
- Modell-Dropdown ist Auswahl ohne Wirkung; Provider (DeepSeek) nicht angebunden.
- Plugins, Scheduled, Site, Connect Mobile, Remote control sowie alle
  Settings-Unterseiten außer "Allgemein": ohne Funktion.
- Browser-Tab: nur UI ("Keine lokalen Server online").
- Menüleiste/Tray, Autostart, Desktop-Notifications: nur persistierte Flags, keine OS-Anbindung.

### Nächste sinnvolle Schritte
1. Agent-Backend anbinden (Sidecar-Muster wie `kilo serve`, HTTP+SSE) + Streaming im MessageList.
2. DeepSeek-Provider + Key-Eingabe in Settings → "Nutzung & Modell".
3. "Öffnen im Editor" verdrahten (`open_workspace_in_editor` existiert) + Download-Fallback prüfen.
4. Optional: Terminal wieder aufnehmen (`TerminalPanel.vue` liegt ungenutzt im Repo).

## 6. Offene Punkte / Risiken

- Detection liefert auf dieser Maschine VSCodium **mehrfach** (nativ `/opt/vscodium-bin`,
  Flatpak, zwei `.desktop`). Dedup + Ranking nötig.
- Zed/Cursor sind installiert, aber **nicht im PATH** → Detection muss `.desktop` lesen.
- macOS/Windows-Pfade in der Detection sind nur Stubs (wir bauen auf Linux).
- WebView-Divergenz (WebKitGTK vs Chromium) ist das spätere Migrationsrisiko für StageVibe.

## 7. Layout-Feinschliff (2026-10-15, nach Nutzer-Review)

Umgesetzt:
- **Sidebars resizeable.** `Resizer.vue` (absolut am Rand positioniert), Store `ui.ts`
  mit `railWidth`/`workWidth` in **% vom Body** + Persistenz (`ds-ide.ui`).
  Default **20 / 60 / 20**.
- **Grenzen:** `RAIL_MIN 12`, `RAIL_MAX 28`, `WORK_MIN 15`, `WORK_MAX 55`, `CENTER_MIN 30`.
- **Auto-Collapse:** wird die rechte Leiste so breit gezogen, dass die Mitte unter
  `CENTER_MIN` fiele, klappt die linke Leiste automatisch ein, um Platz zu schaffen.
- **Nur noch EIN Sidebar-Toggle** — in der `TitleBar`. Entfernt wurden die Toggles im
  LeftRail-Header und in der WorkArea-Tab-Leiste (Dubletten).
- **Weiche Kanten statt Linien:** alle `border: 1px solid var(--border)` zwischen den
  Regionen ersetzt durch `linear-gradient`-Fades (Panel → `var(--bg)`), ~30px
  (TitleBar 18px). Kein harter Übergang mehr.
- `TitleBar`-Padding auf `0 14px` erhöht.
- WorkArea-Sichtbarkeit persistiert (`ds-ide.workarea`).

"Milchglas"-Hinweis: echter Backdrop-Blur bringt nur etwas, wenn dahinter Inhalt liegt;
über solider Fläche ist er unsichtbar. Daher Farbverlauf-Übergang statt Blur.
Echte Fenster-Transluzenz wäre Tauri `windowEffects` (auf Linux/GTK nur eingeschränkt).

## 8. Engine — Quellen gesichtet (2026-10-15)

### Was die Quellen sind
- `minimax-jarvis/` (lokal, **MIT**) = **Definitionsebene, kein Runtime**.
  `agents/` stammt aus `MiniMax-AI/minimax-code` → `packages/local-runtime-v2/assets/agents/`.
  Inhalt: `builtin-agents.json` = `["mavis","explore","worker","verifier"]`; je Agent
  `PERSONA.md` / `PERSONA-zh.md` / `system-prompt.md.hbs`;
  `mavis/modes/{coding,work}/online/{PERSONA,SYSTEM}.md.hbs`;
  `mavis/skills/**/SKILL.md`; `mavis/features/*.md.hbs` (cron, delegation, memory,
  recoverable-deletion, task-management, web-search); `_v2/AGENT_CONTEXT.md.hbs` u. a.
  Die **„Partner"-Rolle steht dort wörtlich**: `description: A partner with judgment and warmth`.
  `tauri-reference/` stammt aus der **closed-source** MiniMax-Desktop-App →
  **nur Ideen/Referenz**, kein Code, keine Assets.
- `MiniMax-AI/skills` (MIT) = **Skill-/Wissenspakete** (frontend-dev, fullstack-dev,
  android/ios/flutter/react-native, shader, pdf/pptx/xlsx/docx, multimodal) für
  Claude Code / Cursor / Codex / OpenCode. Kein Runtime.
- `MiniMax-AI/OpenAgentCore` (MIT) = **serverseitige Infrastruktur**: self-hosted
  OpenAI-Agents-API mit wählbarem Harness (Codex, Claude Code, MiniMax Code), Sandboxes
  (microsandbox/E2B/Docker), `/v1` + `/core/v1`. Braucht Docker/Node → kein Desktop-Baustein.

### Schlussfolgerung
Keine der drei Quellen ist eine einbettbare Engine. Vorhanden ist die **Seele**
(Personas/Prompts/Skills/Modes), **nicht der Motor** (LLM-Loop + Tool-Ausführung).

### Optionen
1. **Eigene schlanke Rust-Engine im Tauri-Prozess** (Empfehlung): DeepSeek
   (OpenAI-kompatibel) + Tool-Loop + Tools (read/write/patch/run), Personas aus
   `minimax-jarvis` (Identitätstexte für unser Produkt umgeschrieben);
   „Full Authorization" → Approval-Gate; Streaming per Tauri-Event in die MessageList.
2. **Sidecar `kilo serve`** (MIT) als fertiger Runtime, Frontend per HTTP+SSE —
   schnellster Weg zu funktionierender Tool-Ausführung.
3. **OpenAgentCore** als Backend (Docker) — mächtig (mehrere Harnesses), aber
   Infrastruktur statt Desktop-Baustein.

### Pflicht bei Übernahme
- MIT-Attribution erhalten (`LICENSE`, `NOTICE`, `THIRD_PARTY_NOTICES.md`).
- Identitätstexte **nicht wörtlich** übernehmen, für unser Produkt umschreiben.
- Rolle: **Partner**, nicht „Agent".

## 9. Engine implementiert (2026-10-15, Rust im Tauri-Prozess)

Entschieden (Nutzer bestätigt): **eigene schlanke Rust-Engine im Tauri-Prozess** —
kein Sidecar, kein Docker. Provider: **nur DeepSeek** (OpenAI-kompatibel).
Partnername: **Seeky** (Ziggy als Alternative, ein String). Rolle: **Partner**.

### Rust — `src-tauri/src/engine/`
- `types.rs` — `EngineConfig` (apiKey, baseUrl, model, systemPrompt, workspace,
  fullAuthorization, maxTurns), `ChatMessage`, `EngineEvent`
  (kind: `stream` | `status` | `tool_start` | `tool_result` | `usage` | `error` | `turn_end`).
  Mapping auf den Mavis-Vertrag (`packages/protocol/src/runtime.ts`):
  `stream`→STREAM_RESP, `status`→SESSION_STATUS, `turn_end`→TURN_TERMINAL;
  `PermissionPolicyType` → unser `fullAuthorization`.
- `deepseek.rs` — Streaming-Client: `POST {baseUrl}/chat/completions`, SSE,
  `stream_options.include_usage`, akkumuliert `tool_calls`-Deltas.
- `tools.rs` — Definitionen + Ausführung: `read`, `write`, `edit`, `bash`, `glob`,
  `grep`, `todowrite`. An MiniMax' `Local*ToolDef` angelehnt.
  **Permission-Gate:** `write`/`edit`/`bash` verweigern ohne Full Authorization.
- `mod.rs` — `EngineState` (pro Session: messages, todos, cancel, seq);
  Kommandos `engine_set_config`, `engine_configured`, `engine_send`, `engine_abort`,
  `engine_reset`; Turn-Loop (max 24 Runden) mit Tool-Runden;
  Events auf Kanal **`engine://event`**.

### Frontend
- `src/engine/persona.ts` — eigene Persona, **nicht** aus den MiniMax-Texten kopiert.
- `src/composables/useEngine.ts` — Event-Listener, `sendPrompt`, `abortTurn`,
  `configureEngine`, `running`, `engineReady`, `lastError`.
- `src/stores/sessions.ts` — Message-Buckets mit Streaming (`beginAssistantMessage`,
  `appendAssistantDelta`, `upsertToolActivity`, `endAssistantMessage`).
- `MessageList` — Text-Stream + Tool-Chips (running/done/failed + Dauer).
- `Composer` — sendet an die Engine, Abort-Button während des Laufs, Modellfeld,
  „Volle Berechtigung"-Toggle (Default **an**).
- Settings → **„Nutzung & Modell"** — API-Key, Basis-URL, Modell.

### Verifiziert
- `cargo check` grün, `cargo test detect_editors_smoke` grün.
- `pnpm build` (vue-tsc + vite) grün.
- `pnpm tauri dev`: Fenster startet, alle Module HTTP 200, `@tauri-apps/api/event`
  pre-bundled ⇒ Frontend hat `initEngine` ausgeführt, keine Rust-Fehler.
- **Live gegen die echte DeepSeek-API grün** (`cargo test --lib live_deepseek --
  --ignored --nocapture`, Key via `DS_IDE_API_KEY`):
  - Text-Streaming: Antwort `PONG`, 4 gestreamte Zeichen, usage 15/2/17.
  - Tool-Calling: `glob` mit `args={"pattern": "Cargo.toml"}` korrekt geparst.

### Offen / bewusst nicht fertig
- Persistiertes `fullAuthorization` steht bei Bestandsinstallationen noch auf `false`
  (Wert von vor der Default-Änderung) → im Composer einmal einschalten, sonst
  verweigern `write`/`edit`/`bash`.
- API-Key liegt im localStorage → Umzug in den OS-Schlüsselbund geplant.
- Approval **blockiert** den Turn nicht: ohne Full Authorization verweigert das Tool
  und der Partner erklärt es. Ein echtes „fragen und warten" (ACTION_REQUIRED-Round-Trip)
  fehlt.
- Nicht portiert: `task*` (Subagenten), `memory`, `skill`, `web_fetch`, `ask_user`,
  `mavis`, `code_review`.
- `pnpm tauri dev` hinterlässt bei hartem Abbruch verwaiste Vite-Prozesse
  (Port 1420 bleibt belegt) → vor Neustart aufräumen.

## 10. UI-Fixes nach der ersten Live-Sitzung (2026-10-15)

Der erste echte Chat-Lauf im Fenster war erfolgreich: Seeky antwortete korrekt und sehr
schnell (Streaming über DeepSeek), und wies die Verwechslung mit „Ziggy" selbst zurück.

Gefixt:
- **Scrollen in beiden Seitenleisten** funktionierte nicht. Ursache: klassischer
  Grid-Fallstrick — eine `auto`-Grid-Zeile wächst über den Container hinaus und wird
  von `overflow: hidden` abgeschnitten, statt zu scrollen. Fix:
  `grid-template-rows: minmax(0, 1fr)` in `App.vue` (`.body`), `ViewFileTab.vue`
  (`.viewfile`) und `SettingsPage.vue` (`.settings`).
- **Dublette** „AGENT-TEAM" in der `LeftRail` entfernt (wurde doppelt gerendert).
- **„Volle Berechtigung"** lag nur im Composer. Jetzt zusätzlich in
  Settings → „Nutzung & Modell" als `ToggleRow` — dort wurde sie gesucht.

### Layout-Umstellung auf Karten (nach Nutzer-Review)

Vorlage: das StageVibe-Layout (dieselbe Designsprache in allen DeepVibe-Produkten).
Entscheidung: **übernommen** — löst das „Abstand"-Problem strukturell statt kosmetisch.

- Neue Variable `--bg-app` (`#141414`, hell `#e4e4e4`) als Fensterhintergrund.
- Globale Klasse `.card` (in `global.css`): `background: var(--bg)`, `1px solid var(--border)`,
  `border-radius: 12px`.
- `App.vue`: `.stage` (Padding) → `.body` (Grid, `gap: 10px`). Die drei Regionen
  (`LeftRail`, `AgentPanel`, `WorkArea`) bekommen `class="card"`.
  Die **Farbverlauf-Fades sind entfernt** — nicht mehr nötig.
- Resizer sitzen jetzt in der Lücke: `left/right: calc(X% + 5px)`.
- `SettingsPage`: nav + content ebenfalls als Karten, `--bg-app` außen.
- Dateibaum im View-File-Tab: `--bg-elevated` gegen das Viewer-`--bg` → der Übergang ist
  jetzt sichtbar statt „ertastbar".

### Weitere Fixes
- **Konto-Menü wurde abgeschnitten:** Ursache war nicht nur der z-index, sondern
  `overflow: hidden` auf `.rail` (clippt absolut positionierte Kinder). `.rail` ist jetzt
  `overflow: visible`, Menü `z-index: 40` (+ Backdrop 39), Breite `min(260px, 100vw - 24px)`.
- `.tree-head`-Selektor war bei einer Reparatur versehentlich gelöscht worden → wiederhergestellt.

## 11. Persistenz: SQLite (2026-10-15)

`rusqlite` mit Feature `bundled` (keine System-Abhängigkeit). Datei:
`<app-data>/com.deepvibe.ds-ide/ds-ide.sqlite3`, **Modus 0600**, WAL, `foreign_keys=ON`.

### Schema (`src-tauri/src/db.rs`)
- `sessions(id, title, grp, workspace, created_at, updated_at)`
- `messages(id, session_id, role, content, tool_calls, tool_call_id, created_at)`
- `tool_calls(id, session_id, message_id, call_id, name, arguments, output, is_error,
  duration_ms, created_at)` — `message_id` hängt den Tool-Call an die Assistant-Nachricht
- `todos(id, session_id, content, status, priority, created_at)`
- `memories(id, kind, scope, text, source, importance, created_at, last_used_at)` — angelegt,
  noch ungenutzt (Langzeitgedächtnis, siehe unten)
- `settings(key, value)` — angelegt, Methoden vorhanden, noch ungenutzt

### Rust
- `Db::open`, `ensure_session`, `touch_session`, `list_sessions`, `create_session`,
  `delete_session`, `insert_message` (liefert rowid), `insert_tool_call`,
  `replace_todos`, `list_todos`, `load_history` (mit verschachtelten Tools),
  `load_llm_messages` (rekonstruiert die LLM-Historie inkl. `tool_calls`),
  `set_setting`/`get_setting`.
- Kommandos: `sessions_list`, `session_create`, `session_delete`, `session_history`,
  `session_todos`.
- **Umbau der Engine:** Die Konversation ist **nicht mehr nur im Speicher**. `engine_send`
  schreibt die User-Nachricht **sofort**, lädt die Historie **aus der DB** für den LLM-Call
  und schreibt Assistant-Nachricht + jede Tool-Ausführung durch. `todowrite` persistiert
  die Todo-Liste. `engine_reset` löscht die Session-Inhalte in der DB.
  `SessionState` hält nur noch `cancel` + `seq`.

### Frontend
- `useEngine`: `loadSessions`, `loadHistory`, `openSession`, `createSession`;
  `initEngine` lädt Workspace → Config → Sessions → Historie.
- LeftRail: „Neue Aufgabe" legt echt eine Session an; Session-Klick lädt die Historie.
- Die hartkodierte Session-Liste ist ersetzt — sie kommt aus der DB.

### Verifiziert
- `cargo check` grün; Unit-Test `db::tests::round_trips_history_and_todos` grün
  (Session, Nachrichten, Tool-Anhang, Todos, LLM-Historie, Session-Liste).
- `pnpm build` grün.
- `pnpm tauri dev`: DB wird angelegt (`-rw-------`), alle sechs Tabellen, Frontend legt
  beim Start die erste Session an (`s1792061157035`, workspace `/home/me`).

### Bewusst NICHT gemacht
- Der **API-Key liegt weiter im localStorage** — er gehört nicht in die DB. Umzug in den
  OS-Schlüsselbund ist der nächste Schritt (siehe Abschnitt 12).

## 12. Entscheidungen: Secrets & Langzeitgedächtnis (2026-10-15)

### Secrets (API-Key)
- **Nicht in die SQLite-DB.** Klartext-Secrets in einer Anwendungs-DB sind der falsche Ort —
  und eine verschlüsselte DB bräuchte wieder ein Bootstrapping-Geheimnis.
- Ziel: **OS-Schlüsselbund** via `keyring`-Crate (Secret Service / Keychain / Credential
  Manager). Auf dieser Maschine verfügbar: `gnome-keyring` läuft, `org.freedesktop.secrets`
  ist auf dem Session-Bus aktiv, libsecret 0.21.8.
- Damit verlässt der Key das Frontend **komplett**: `engine_set_config` bekommt keinen
  `apiKey` mehr, Rust lädt ihn beim Senden aus dem Keyring. Das Frontend kennt nur noch
  `hasApiKey`.
- Fallback ohne laufenden Keyring: `<app-data>/secrets.json` mit Modus 0600 **plus
  sichtbarer Warnung** in den Settings — kein stiller Klartext.
- `SQLCipher` (verschlüsselte DB) bewusst **nicht**: löst das Key-Problem nicht, verschiebt es.
- Nicht-Secrets (Modell, baseUrl, Appearance, Toggles, Workspace) → `settings`-Tabelle,
  damit localStorage nur noch UI-Kram hält.

### Langzeitgedächtnis („aus Persona wird Persönlichkeit")
1. **Vorbereitet:** Tabelle `memories` (kind, scope, text, source, importance, created_at,
   last_used_at).
2. **Stufe 1 — bewusstes Merken:** ein `memory`-Tool (existiert im MiniMax-Tool-Set), mit dem
   der Partner explizit Fakten/Präferenzen/Ereignisse ablegt — **nicht** automatisch alles
   mitschneiden. Dazu eine Review-Oberfläche (ansehen, korrigieren, löschen).
3. **Stufe 2 — Erinnern:** beim Turn-Start die relevantesten Einträge in den Systemprompt
   injizieren (scope `user`/`agent`, gewichtet nach `importance` + `last_used_at`).
4. **Stufe 3 — semantische Suche:** **Qdrant** als Vektorindex, gefüttert aus SQLite.
   Einbahnig: **SQLite bleibt die Quelle der Wahrheit**, Qdrant ist nur Suchindex und
   jederzeit neu aufbaubar.
5. **Kuration („der Doktor"):** periodischer Lauf, der Memories zusammenfasst, Widersprüche
   und Veraltetes markiert und aufräumt. Genau dafür ist MiniMax' `mavis-doctor` gedacht —
   sein Inhalt ist in der öffentlichen Source **nicht enthalten**, also bauen wir ihn selbst.

`kind`: `fact`, `preference`, `event`, `project`, `news`.
`scope`: `user` (über den Nutzer), `agent` (über Seeky selbst), `session` (flüchtig).

### Weitere Mavis-Übernahme (Kandidaten)
Neben der Persona lohnen als Nächstes: `mavis/features/*.md.hbs` (cron, delegation, memory,
recoverable-deletion, task-management, web-search) und `_v2/AGENT_CONTEXT.md.hbs`
(Harness-/Tool-Usage-/Memory-/Output-Konventionen).

## 13. Backlog — Abarbeitungsreihenfolge (2026-10-15)

Nutzerauftrag: „so gut wie eigenständig abarbeiten, außer bei Überprüfungen".
Nach **jedem** Punkt: `pnpm build`, `cargo test`, PROGRESS.md aktualisieren.

**Stand 2026-10-15 (zweite Runde): Punkte 1-9 sowie 12, 13, 14, 15 sind erledigt und
verifiziert. Offen ist nur noch 10 (Walfisch-Branding — wartet auf das Nutzer-Asset).
Alle Commits sind auf `origin/main`.**

Verifikation dieser Runde: `pnpm build` grün · `cargo test` 9 passed / 8 ignored ·
alle Live-Tests grün (`list_local_skills_smoke`, `live_code_review_runs_git`,
`live_web_fetch_reads_example_com`, zwei Subagenten-Tests, drei DeepSeek-Live-Tests;
Key aus dem OS-Schlüsselbund).

1. ✓ **Autoscroll im AgentPanel** — erledigt. `AgentPanel.vue`: rAF-gedrosseltes Nachführen
   der Scrollposition, `following`-Flag aus einer 48px-Bodenzone, „Nach unten"-Knopf
   (i18n `composer.scrollToBottom`) erscheint nur, wenn pausiert; eine eigene
   User-Nachricht setzt `following` wieder auf `true`. Session-Wechsel springt ans Ende.
   Evidenz: `pnpm build` grün · `cargo test` 2 passed / 2 ignored · `pnpm tauri dev`
   startet, Modul HTTP 200.
2. ✓ **Secrets in den OS-Schlüsselbund** — erledigt.
   - `src-tauri/src/secret.rs`: `SecretStore` mit Keyring-first und 0600-Datei-Fallback,
     `SecretStatus { present, backend: keyring|file|none }`; Kommandos `secret_status`,
     `secret_set`, `secret_clear`.
   - `EngineConfig.api_key` ist jetzt `#[serde(skip, default)]` → **das Frontend kann keinen
     Key mehr einschleusen**. `engine_send` liest ihn pro Turn aus dem Keyring.
   - Frontend: `settings.apiKey` **entfernt**; Settings-Seite hat Eingabefeld + Speichern/
     Entfernen und zeigt den Backend-Status; bei `file` erscheint eine **rote Warnung**.
   - Automatische **Migration**: ein noch im localStorage liegender Key wird einmalig
     in den Schlüsselbund geschrieben und danach aus localStorage gelöscht.
   - Evidenz: `cargo test --lib secret` → `keyring_roundtrip_or_skip` **ok** (kein Skip,
     also echter Secret-Service-Zugriff); `pnpm build` grün; nach `pnpm tauri dev`:
     `secret-tool search service ds-ide.deepvibe` findet den Eintrag (Ausgabe verworfen,
     Key wurde nie ausgegeben), `apiKey` ist aus localStorage verschwunden,
     keine `secrets.json` vorhanden.
3. ✓ **`settings`-Tabelle gefüllt** — erledigt. `db.rs`: `all_settings` + Kommandos
   `settings_all` / `settings_set`. Der Settings-Store (`stores/settings.ts`) liest beim
   Start aus SQLite und schreibt **entprellt** (400 ms) zurück; ein alter
   `ds-ide.settings`-Blob aus localStorage wird einmalig importiert und gelöscht.
   `main.ts` mountet die App erst nach dem Laden (kein Theme-/Sprach-Flackern).
   Evidenz: nach `pnpm tauri dev` stehen **11 Keys** in `settings`
   (u. a. `mode: coding`, `model: deepseek-flash`, `pinnedEditorId: zed`,
   `workspace: /home/me`); localStorage enthält nur noch `ds-ide.ui`,
   `ds-ide.ui.v2`, `ds-ide.workarea`. `pnpm build` grün, `cargo test` 3 passed.
4. ✓ **Reasoning sichtbar** — erledigt. `deepseek.rs` liefert jetzt `DeltaKind::{Text, Reasoning}`
   an den Callback (vorher wurde `reasoning_content` **verworfen**); `StreamOutcome` trägt
   `reasoning`. Die Engine emittiert dafür Events mit `kind = "reasoning"`, das Frontend
   hängt sie an `ChatMessage.reasoning` und zeigt sie als **einklappbares `<details>`**
   über der Antwort (offen, solange noch kein Text da ist).
   Reasoning wird **nicht** persistiert (flüchtiges Denken; die DB soll keine Denkspuren
   sammeln) — beim Neuladen steht die Antwort, nicht das Nachdenken.
   Evidenz: neuer Live-Test `live_deepseek_reasoner_streams_thinking` grün —
   `reasoning chars: 166`, Antwort `391`; `pnpm build` grün, `cargo test` 3 passed.
5. ✓ **Langzeitgedächtnis Stufe 1+2** — erledigt (Stufe 3/5 = Punkt 6).
   - `db.rs`: `insert_memory`, `list_memories`, `memory_context` (sortiert nach
     `importance`, dann `last_used_at`, dann Alter), `touch_memories`, `delete_memory`;
     Kommandos `memories_list`, `memory_delete`.
     *Überholt (Abschnitt 21):* `memory_delete` gibt es nicht mehr, Löschen heißt jetzt
     Stilllegen (`memory_retire`/`memory_restore`), und Einträge tragen `origin` und `edited_at`.
   - Tool **`memory`**: `action` add|list|forget, `kind` fact|preference|event|project|news,
     `scope` user|agent|session. Ausführung liegt bewusst in `engine/mod.rs` (`memory_tool`),
     damit `tools.rs` speicherfrei bleibt.
   - **Erinnern:** `run_turn` hängt vor jedem Turn einen Block „What you already know"
     (bis 40 Einträge) an den Systemprompt und stempelt `last_used_at`.
   - **UI:** neuer Work-Area-Tab **„Gedächtnis"** (`MemoryTab.vue`, Toolbar-Button mit
     Sparkle-Icon) — Einträge mit Scope/Kind/Datum und Entfernen-Knopf.
     *Überholt (Abschnitt 21):* der Tab ist weg, die Einsicht sitzt in Einstellungen →
     Personalisierung (`MemoryPanel.vue`).
   - Rationale: `kind`/`scope` statt Freitext, damit die Kuration (Punkt 6) etwas zu
     greifen hat.
   - Evidenz: `cargo test` → `db::tests::memories_round_trip` ok (insert/list/context/
     touch/delete); `pnpm build` grün; `pnpm tauri dev` startet, Module HTTP 200;
     **keine Rust-Warnungen** mehr.
6. ✓ **Kuration / „Doktor"** — erledigt, aber **bewusst mechanisch**.
   - `db.rs`: `memory_report(stale_after_days)` liefert `MemoryReport { total, duplicates,
     stale }`. Duplikate = gleicher Text nach Normalisierung (Whitespace + Groß/Klein);
     behalten wird der älteste Eintrag. `stale` = **nie benutzt** und älter als die Frist
     (30 Tage). Kommandos `memory_report`, `memory_prune(ids)`.
     *Überholt (Abschnitt 21):* `memory_prune` löscht nichts mehr, es legt still.
   - Tool `memory` hat jetzt `action: "curate"` — der Partner kann den Bericht selbst
     anfordern und dann **gezielt** mergen oder vergessen.
   - UI: im Gedächtnis-Tab ein **Prüfen**-Knopf (Sparkle). Der Bericht erscheint als Karte
     mit „N doppelte Einträge" / „N nie benutzte, alte Einträge" und je einem
     **Entfernen**-Knopf. Ohne Befund: „Nichts zu bereinigen."
   - **Entscheidung:** Der „Doktor" schlägt vor, **löscht aber nie von selbst**. Was
     *inhaltlich* zusammengehört, kann eine Heuristik nicht beurteilen — das bleibt beim
     Modell (per `curate`) oder beim Nutzer. Ein Gedächtnis, das sich selbst ungefragt
     aufräumt, ist genau die Sorte Werkzeug, der man nicht mehr traut.
   - Evidenz: `cargo test` → `db::tests::memory_report_flags_exact_duplicates` ok
     (Erkennung trotz abweichender Leerzeichen/Groß-Klein-Schreibung, frische Einträge
     nicht als stale); `pnpm build` grün (9 Tests grün, 6 ignoriert); App startet,
     Modul HTTP 200.
7. ✓ **Weitere Tools** — erledigt: `web_fetch`, `code_review`, `ask_user`, `task`.
   - **`web_fetch`** — holt eine http(s)-URL (30 s Timeout), `html_to_text` entfernt
     Script-/Style-Blöcke, Tags und dekodiert die gängigen Entities, Ausgabe gekappt.
     Bewusst ohne DOM-Parser: dependency-frei und vorhersagbar.
   - **`code_review`** — sammelt `git diff --stat` + `git diff` (optional `--cached`,
     optional Pfad) im Workspace und gibt beides zurück. Ehrlich: es *holt* den Diff,
     die Bewertung macht das Modell.
   - **`ask_user`** — **echter** `ACTION_REQUIRED`-Round-Trip statt Attrappe: der Turn
     emittiert ein Event mit Frage und optionalen Optionen, parkt einen `oneshot::Sender`
     in `EngineState.pending` und **wartet** (Timeout 5 min). Das Kommando
     `engine_answer(actionId, answer)` löst ihn auf; im Chat erscheint über dem Composer
     eine Rückfrage-Karte mit Optionsknöpfen und freiem Feld.
   - **`task` (Subagent)** — eigener Kontext, eigene Tool-Runden (max. 8), liefert **nur das
     Endergebnis** zurück (gekappt auf 20 000 Zeichen). Sein Verlauf erscheint **nicht** im
     Chat, sondern nur die Tool-Karte — genau darum geht es: den Elternkontext sauber halten.
     Absichtliche Schutzgeländer: **kein** verschachteltes `task` (keine Rekursion) und
     **kein** `ask_user` im Subagenten (er könnte den Nutzer über die Eltern-UI nicht
     erreichen). `task_tool` braucht **kein** `AppHandle` — dadurch ist es direkt testbar.
   - Evidenz: `cargo test` → `html_to_text_drops_scripts_styles_and_tags` ok; mit
     `--ignored`: `live_web_fetch_reads_example_com` ok (184 Bytes Text),
     `live_code_review_runs_git` ok (git-Diff im Arbeitsbaum),
     `live_task_subagent_answers_without_tools` → „BEREIT",
     `live_task_subagent_uses_its_tools` → Subagent zählte per `glob` **150** Einträge und
     gab nur die Zahl zurück; `pnpm build` grün; App startet.
   - **Nicht verifiziert:** ein echter `ask_user`-Durchlauf braucht eine Antwort im UI —
     der Pfad ist gebaut und kompiliert, aber noch nicht von Hand durchgeklickt.
8. ✓ **„Suche" verdrahtet, Rest ehrlich gekennzeichnet** — erledigt.
   - Rust: `Db::search` + `snippet_around` (Kontext-Ausschnitt um den Treffer) und Kommando
     `search` — durchsucht Sitzungstitel und Nachrichteninhalte (`LIKE`, Limit 40).
   - UI: `SearchDialog.vue` als Overlay (Teleport), öffnet über „Suchen" in der Leiste
     **oder Strg/Cmd+K**, Esc schließt, Enter öffnet den ersten Treffer. Treffer sind als
     „Sitzung"/„Nachricht" gekennzeichnet, Klick öffnet die Sitzung.
   - Ehrlichkeit: „Skills", „Geplant" und „Seite" tragen jetzt ein **„bald"**-Abzeichen,
     damit die Leiste nicht mehr suggeriert, sie würden schon funktionieren.
   - Evidenz: `cargo test` → `db::tests::search_finds_sessions_and_messages` ok
     (Titeltreffer, Nachrichtentreffer mit Snippet, leere Query bleibt leer);
     `pnpm build` grün; App startet, Module HTTP 200.
9. ✓ **Fortschritts-Tab aus echten Todos** — erledigt. Die hartkodierten
   „Liefergegenstände" aus `stores/sessions.ts` sind **entfernt**; der Tab liest jetzt
   `session_todos` (Rust liefert `TodoRow { id, content, status, priority }` statt nur
   Strings) und gruppiert nach Status — „In Arbeit / Offen / Erledigt", Erledigtes
   durchgestrichen. Aktualisiert sich live: `useEngine` zählt `todosRevision` hoch, sobald
   ein `todowrite`-Ergebnis eintrifft.
   Evidenz: `cargo test` grün (Test prüft `list_todos()[0].content`), `pnpm build` grün,
   App startet, Module HTTP 200, `todos`-Query liefert die erwartete Spaltenform.
10. ◐ **Walfisch-Branding** — statischer Teil erledigt, Animation offen.
    - Quelle gefunden: `~/Schreibtisch/ziggy.png` (512×512, **bereits korrekt freigestellt**:
      Ecken voll transparent, mittlere Deckkraft 0,33) plus `ziggy1.webp`, `ziggy.webp`.
    - **App-Icon-Set erzeugt**: `ziggy.png` getrimmt, auf 900 px skaliert, mittig auf
      1024×1024 gepolstert → `pnpm tauri icon` → `src-tauri/icons/*` (inkl. Android/iOS)
      aktualisiert.
    - **Avatar im Chat**: `public/whale.png` (128×128, transparent), in `MessageList.vue`
      links neben Assistant-Nachrichten (22 px). Liegt auch im Build (`dist/whale.png`).
    - **Animation offen — und der Grund ist echt:** Die Videovorlage
      (`~/Downloads/ai-artwork-video_*.mp4`, 752×560) hat einen **blauen Verlaufs­hintergrund**
      (Ecken von `rgb(56,164,212)` bis `rgb(171,238,255)`), und der Wal ist **ebenfalls blau**
      inklusive hellem Bauch. Farb-Keying zerstört ihn deshalb — genau daran scheitern die
      Web-Tools. Sauber geht das nur mit **Segmentierung** (z. B. `rembg`/U²-Net), nicht mit
      `colorkey`. Das braucht ein Modell (~170 MB Download).
    - Nächster Schritt, wenn gewünscht: `rembg` in einer venv installieren, die 97 Frames
      freistellen und als **animiertes WebP** (Alpha, viel kleiner als GIF) für Splash und
      Chat-Avatar ausgeben. Fenster-/Taskleisten-Icon bleibt statisch — kein OS animiert
      App-Icons.
11. **Regelmäßig**: `pnpm build` + `cargo test` + Verifikation im Fenster + PROGRESS.md.

### Bewusste Grenzen in dieser Runde
- Kein Approval-Round-Trip (`ACTION_REQUIRED` fragt und wartet) — das Gate verweigert nur.
- Keine Subagenten, keine Plugin-/MCP-Anbindung, kein Web-Browsing-Tab im Betrieb.

## 14. UI-Bereinigung: was ist echt, was ist MiniMax-Erbe (2026-10-15)

Der Nutzer hat gefragt, welche der übernommenen Nav-Einträge wirklich umsetzbar sind.
Befund und Entscheidung:

| Eintrag | Herkunft | Entscheidung |
|---|---|---|
| Mobil verbinden | MiniMax-Companion-App (Android/iOS, „Mobile app 4.15.0+") | **raus** — wir haben keine Companion-App |
| Fernsteuerung | MiniMax-Fernsteuerung eines laufenden Tasks | **raus** (gleicher Grund) |
| Plugins | MiniMax-Marketplace (eigene Accounts, Usage-Zähler) | **kein Store** — DeepSeek hat keinen. Stattdessen **lokale Skills** |
| Geplant | MiniMax-Scheduler („keep awake") | **umsetzbar**, eigenes Item |
| Seite | reiner Prompt-Aufruf („Create a website") | als **Skill** übernehmen (`deploy-website`, `visual-page`) |
| Abonnieren | MiniMax-Abo | **streichen** — DeepSeek hat nur **Top-up (Credits)**, kein Abo |
| Täglicher Check-in | MiniMax Free-Tier-Feature | **streichen** |
| Nutzung / Feedback / Änderungsprotokoll / Kontakt / Mehr erfahren / Abmelden | gemischt (DeepSeek-Plattform + unseres) | später verlinken |

**Skills-Quellen:** `https://github.com/MiniMax-AI/skills` (MIT, generische
Markdown-Anleitungen, kein Store nötig). Das README dort nennt die Einbaupfade für
Claude Code, Cursor, Codex und OpenCode. Für DS.IDE: eigenes Verzeichnis
`~/.ds-ide/skills/**/SKILL.md` + Loader.

### Zusätzliche Backlog-Punkte (nach 10)
12. ✓ **Nav bereinigt** — erledigt. Entfernt: „Mobil verbinden" und „Fernsteuerung"
    (brauchen MiniMax' Companion-App), „Abonnieren" (DeepSeek hat nur Top-up-Credits) und
    „Täglicher Check-in" (MiniMax-Free-Tier). „Plugins" → **„Skills"**. Übrig bleiben
    „Neue Aufgabe" (funktioniert), „Suche" (Punkt 8), „Skills" (14), „Geplant" (13),
    „Seite" (14) — die letzten drei bewusst als geplante Ziele, nicht als Attrappen.
    Evidenz: `pnpm build` grün (vue-tsc prüft die i18n-Kataloge gegen die Form-Referenz,
    also sind beide Sprachen konsistent).
13. ✓ **Eigener Scheduler** — erledigt (bewusst **Intervall**, nicht Cron).
    - Tabelle `schedules` (session_id, prompt, interval_seconds, enabled, last_run_at,
      next_run_at). Das Schema läuft bei jedem Öffnen — die Tabelle landete also **ohne
      Migration** in der bestehenden DB.
    - `db.rs`: `list_schedules`, `create_schedule` (Minimum 60 s), `delete_schedule`,
      `set_schedule_enabled`, `due_schedules(at)`, `mark_schedule_run`; Kommandos
      `schedules_list`, `schedule_create`, `schedule_delete`, `schedule_toggle`.
    - Engine: `engine_send` wurde zu **`send_prompt(app, session_id, prompt)`** extrahiert,
      der Befehl ist nur noch eine Hülle. Dadurch läuft ein geplanter Auftrag durch **genau
      denselben Pfad** wie ein getippter (Persistenz, Tools, Events).
    - Hintergrundtask im `setup` von `lib.rs`: alle 30 s `run_due_schedules()`, das fällige
      Einträge startet und `next_run_at` fortschreibt.
    - UI: Tab **„Geplant"** (`SchedulesTab.vue`) — Auftrag + Intervall in Minuten anlegen,
      ein-/ausschalten, löschen, nächster und letzter Lauf. „Geplant" in der Leiste öffnet
      den Tab, das „bald"-Abzeichen ist weg.
    - **Ehrliche Grenzen:** Intervall statt Cron-Ausdrücken, und der Task lebt **nur, solange
      DS.IDE läuft** — kein Daemon, kein Aufwecken aus dem Ruhezustand. „keep awake" habe ich
      deshalb bewusst **nicht** gebaut, statt es als Schalter ohne Wirkung anzubieten.
    - Evidenz: `cargo test` → `db::tests::schedules_cycle` ok (nicht fällig → fällig →
      fortgeschrieben → deaktiviert → gelöscht); `pnpm build` grün; nach `pnpm tauri dev`
      existiert `schedules` in der bestehenden DB (0 Zeilen), Modul HTTP 200.
14. ✓ **Skills lokal** — erledigt.
    - `src-tauri/src/skills.rs`: scannt `~/.ds-ide/skills/**/SKILL.md` (Tiefe 3), liest
      YAML-Frontmatter (`name`, `description`) tolerant — ohne `name` fällt es auf den
      Verzeichnisnamen zurück, unlesbare Dateien werden übersprungen statt die Liste zu
      sprengen. Kommandos `skills_list`, `skills_root_path` (legt den Ordner an).
    - Tool **`skill`**: `action: list | load`, `name`. `load` liefert die vollständige
      SKILL.md als Tool-Ergebnis, damit der Partner sie befolgen kann. Liegt in `tools.rs`
      (nur Dateisystem); der Pfad kommt über `ToolContext.skills_root`.
    - UI: neuer Work-Area-Tab **„Skills"** (`SkillsTab.vue`) mit Name, Beschreibung, Pfad,
      „Ordner öffnen" (`revealItemInDir`) und Neu laden. „Skills" in der linken Leiste
      öffnet den Tab — das „bald"-Abzeichen ist dort weg.
    - Beispiel-Skill angelegt: `~/.ds-ide/skills/release-notes/SKILL.md` — damit der Tab
      nicht leer startet und das Format sichtbar ist.
    - Evidenz: `cargo test` → Frontmatter-Test **und** `list_local_skills_smoke`
      (bewusst `#[ignore]`, weil vom Home abhängig) findet real
      `release-notes — Turn a git log into short, human release notes.`;
      `pnpm build` grün, App startet, Modul HTTP 200.
15. ✓ **Dialog-Buttons platzsparend** — API-Key-Zeile: Textbuttons durch **✅ / ❌**
    ersetzt (Tooltips „Speichern"/„Entfernen"), Eingabefeld flexibel statt fix 320px
    (lief vorher rechts aus dem Panel).

## 15. Repository (2026-10-15)

TEST ist ein **eigenes Git-Repo** (vorher lag es nur untracked im noch leeren
`~/Projects`-Repo).
- Remote: `http://192.168.0.40/modest/DS.IDE.git`, Branch `main`
- Erster Commit: `2d1a5d2` „first commit", 82 Dateien, Push erfolgreich (Upstream gesetzt)
- `.gitignore` erweitert: `kilocode-main/`, `minimax-code-main/`, `minimax-jarvis/`,
  `monaco-editor-0.57.0/` (nur lokale Referenz) und `src-tauri/secrets.json`.
  `src-tauri/target/` ist über `src-tauri/.gitignore` abgedeckt, `node_modules`/`dist`
  über die Wurzel. **Nichts davon ist im Repo** (geprüft: 0 Treffer).
- Kein Secret im Quellcode oder in den getrackten Dateien.
- Hinweis: Weil `~/Projects` selbst ein (noch leeres) Repo ist, sieht dieses beim Commit
  `TEST` als eingebettetes Repo (Gitlink) — gewollt, aber gut zu wissen.

## 16. Sitzungen: Titel, Anheften, Umbenennen (2026-10-15)

Auf Nutzerwunsch (MiniMax als Vorbild, aber bewusst schlanker):
- **Titel aus der ersten Nachricht.** `maybe_generate_title` schreibt sofort einen
  Text-Fallback (erste Zeile, 60 Zeichen) und lässt dann im Hintergrund einen kurzen
  Modellaufruf („drei bis sechs Wörter, keine Anführungszeichen") einen besseren Titel
  setzen. Beide Schritte schreiben in die DB und schicken ein `title`-Event, damit die
  Leiste sofort nachzieht. Der Titel wird **nur** gesetzt, wenn er noch leer ist — ein
  selbst umbenannter Titel wird nie überschrieben.
- **Kontextmenü in der Leiste** (Rechtsklick auf eine Sitzung): **nur Anheften/Abheften
  und Umbenennen** — nicht die MiniMax-Liste (Archive, Fork, Show in Finder, Copy …).
  Bewusste Entscheidung: wir zeigen nur, was auch funktioniert.
- **Umbenennen** passiert **inline** in der Zeile (kein `window.prompt`): Enter übernimmt,
  Esc bricht ab, Blur übernimmt.
- Rust: `session_title`, `update_session(id, title?, group?)`, Kommando `session_update`.
- Evidenz: `cargo test` → `db::tests::sessions_can_be_renamed_and_pinned` ok (leerer Titel
  → Umbenennen → Anheften); `pnpm build` grün; Module gegen die laufende Dev-Instanz
  HTTP 200 (inkl. `/whale.png`).

### Lernnotiz zum Ablauf
`pnpm tauri dev` startet das App-Binary **auch dann**, wenn der `beforeDevCommand` am
belegten Port 1420 scheitert. Dadurch entstanden mehrfach zusätzliche Fenster, die einzeln
aufgeräumt werden mussten. Künftig: **nicht booten, wenn 1420 belegt ist**, sondern die
Module gegen die laufende Instanz prüfen.

## 17. Archiv, Konto, Wal-Animation (2026-10-15)

### Archivieren statt löschen
- `sessions.archived` per **Migration** ergänzt: `migrate()` prüft `PRAGMA table_info`
  und macht bei Bedarf `ALTER TABLE` — `CREATE TABLE IF NOT EXISTS` ändert bestehende
  Tabellen nicht. Auf der echten DB verifiziert (Spalte da, beide Sitzungen erhalten).
- `list_sessions` blendet Archiviertes aus, `list_archived_sessions` zeigt es; Kommandos
  `session_archive`, `sessions_archived`.
- UI: Kontextmenü → **Archivieren**; Settings → **Archivierte Aufgaben** listet sie mit
  **Reaktivieren**. Der Verlauf bleibt vollständig in der DB.
- Evidenz: `archiving_hides_a_session_without_deleting_it` und
  `migrate_adds_archived_to_an_old_database` grün.

### Konto
- **Anzeigename ist keine Attrappe mehr:** Setting `displayName`, Vorgabe vom System
  (`default_display_name` liest `$USER`/`$USERNAME`/`$LOGNAME`), einstellbar in
  Settings → Konto. Die erfundene „UID" aus dem MiniMax-Screenshot ist **entfernt**.
- **Menü:** Einstellungen (lokal) · Konto + Nutzung → DeepSeek · Feedback → GitHub ·
  Kontakt → Issues · Mehr erfahren → Projektseite · **Anmelden/Abmelden** je nach
  Key-Zustand (Abmelden löscht den Key aus dem Schlüsselbund, Anmelden springt direkt in
  Settings → Nutzung & Modell). „Änderungsprotokoll" entfernt (hatte kein Ziel).
- **Alle externen Adressen stehen in `src/engine/links.ts`** — eine Datei, klar
  kommentiert. `issues` zeigt vorläufig auf die Org-Seite, weil es noch kein öffentliches
  Repo gibt; dort bitte auf das echte Repo zeigen.

### Wal-Animation
- `rembg` in `~/.local/share/ds-ide/tools/venv-rembg` installiert (Freigabe des Nutzers).
  Achtung: die aktuelle Version lädt standardmäßig **bria-rmbg (1,02 GB)**, nicht u2net.
- **Wichtig fürs Tempo:** `rembg i` per CLI lädt das Modell **bei jedem Aufruf neu**
  (~22 s/Frame). Ein Python-Lauf mit `new_session()` einmal laden und dann alle Frames
  durchziehen ist um ein Vielfaches schneller (`/tmp/kilo/whale/cut.py`).
- 40 Frames (10 fps aus dem MP4) freigestellt, **fester gemeinsamer Zuschnitt**
  636×470+66+55 (aus dem Union-Bounding-Box aller Frames — per-Frame-Trim würde den Wal
  springen lassen).
- Ergebnis: **animiertes WebP mit Alpha**, `public/whale-anim.webp` (128 px, 152 KB) und
  `public/whale-splash.webp` (256 px, 332 KB), je 40 Frames.
- **Eingebaut:** Start-Animation (Splash-Overlay, blendet nach ~1,6 s aus) und im Chat
  schwimmt der Wal **statt der drei Blinkpunkte**, während auf die Antwort gewartet wird.
  Das Fenster-/Taskleisten-Icon bleibt statisch — kein OS animiert App-Icons.
- Statisch war schon fertig: Icon-Set aus `ziggy.png`, Chat-Avatar `public/whale.png`.
- Stolperstein: Ein abgebrochener Batch hinterließ `f020-cut.png` mit **0 Bytes**; ffmpeg
  bricht an so einer Lücke ab und lieferte nur 19 statt 40 Frames. Prüfen mit
  `find … -size 0`.

### Lernnotiz zsh
`rm -f verzeichnis/*-glob.png` bricht in zsh bei **leerem Glob** mit „no matches found" ab
und reißt per `&&` die ganze Kette mit. In Skripten `find … -delete` verwenden oder
`setopt null_glob`.

## 18. Sprachen, Links, Walfisch-Marke (2026-10-15)

### i18n auf sechs Sprachen
Ausgelagert an **vier Hintergrund-Subagenten** (Vorschlag des Nutzers), je eine Datei:
`zh-Hans` (Pflicht — Muttersprache des Betreibers), `fr`, `es`, `ru`. Registriert in
`i18n/index.ts` samt lesbarer Sprachliste (Sprachnamen als eigene Labels: 简体中文,
Русский, Français, Español). `detectLocale` erkennt jetzt auch `zh` und `ru`.

**Lehre — die Selbstauskunft der Subagenten war nicht belastbar.** Drei von vier
lieferten fehlerhafte Dateien, obwohl alle „KEY-SET IDENTISCH" meldeten:
- `fr` und `es` hatten einen **Syntaxfehler**: deutsche Anführungszeichen `„…"` mitten in
  einem doppelt gequoteten TS-String — das schließende `"` beendet den String.
- `fr` fehlten am Ende **fünf Keys** (`save`, `clear`, `notSet`, `storedKeyring`,
  `storedFile`).
- `zh-Hans` und `ru` hatten **deutsche Reste** in den Werten.
Was geholfen hat, sind zwei eigene Prüfungen (stehen als Rezept hier):
1. **Keys:** `pnpm build` — `vue-tsc` erzwingt über den Typ `Messages` die exakte
   Key-Gleichheit und nennt fehlende Felder beim Namen.
2. **Werte:** identische `key: "value"`-Zeilen zwischen Quelle und Übersetzung zählen
   (`comm -12`). Was übrig bleibt, ist entweder eine Marke (`DS.IDE`, `DeepSeek`, `Seeky`)
   oder eine übersehene Stelle. So fielen `tagline`, `providerDesc`, `modeEverydayDesc`
   und ein deutsches „OpenAI-kompatibel" in `es` auf.
Merksatz: Subagenten erzeugen Text, aber **verifizieren muss man selbst** — ihre
Bestätigung ist kein Nachweis.

### Externe Adressen
Alle in `src/engine/links.ts`: Konto → `platform.deepseek.com/profile`, Nutzung → `/usage`,
Feedback → `github.com/moc72/dside/issues`, Kontakt → `mailto:me@modestcoders.eu`,
Mehr erfahren → `modestcoders.eu`.

### Oberfläche
- Der Wal sitzt jetzt als **kleines animiertes Markenzeichen (26 px)** oben links in der
  Leiste, wo „DS.IDE" stand; im Chat sind wieder die drei Punkte (er störte dort).
  Der Splash beim Start bleibt.
- Einstellungen ausgedünnt: **Connection**, **Code review** und **Worktree** entfernt
  (MiniMax-Erbe ohne Substanz), **„Basis-URL" aus der Oberfläche** genommen.
- **Modelle kommen von der API** (`GET /models`, Knopf am Modellfeld) statt geratener Namen.
- **„Abmelden" ist jetzt zweistufig**: erst fragen (rot, ❌, „Entfernen"), dann löschen.
  Auslöser war der Nutzer, der es ausprobierte und dabei seinen Key verlor — das Verhalten
  war korrekt, aber zu scharf.

## 19. Vorschauen (2026-10-15)

### Was läuft
- **Markdown** — `marked` + **DOMPurify**, Umschalter Vorschau/Quelle. Markdown erlaubt
  rohes HTML; ohne Säuberung könnte eine Notizdatei Skript in unserem Webview ausführen,
  das Tauri-Befehle aufrufen darf.
- **Bilder** — über `read_file_base64` als Data-URL.
- **PDF** — `pdfjs-dist` 6.4, Seite für Seite auf Canvas, Worker gebündelt
  (`pdf.worker.min.mjs`, 1,26 MB). v6 will `render({ canvas, viewport })` statt
  `canvasContext`.
- Weiche: `previewKindFor()` in `src/lib/preview.ts`.

### Zwei Fehler, die wir gefunden haben
1. **Das Asset-Protokoll ist blockiert.** WebKit lehnt `asset://localhost/...` von der
   Dev-Server-Origin ab: „protocols and ports must match". Bilder über `convertFileSrc`
   liefen deshalb ins Leere. **Das gilt auch für die geplante HTML-Vorschau** — der
   Base64-Weg ist die verlässliche Variante. Asset-Protokoll und das Cargo-Feature
   `protocol-asset` sind wieder entfernt.
2. **Der Text-Loader lief für jede Datei.** `read_text_file` scheitert an Binärdaten
   (kein UTF-8) und setzte `error` — über einer Vorschau, die gerade lud. Ergebnis:
   „Datei konnte nicht gelesen werden" bei Bildern und PDFs. Jetzt entscheidet
   `previewKindFor`, ob überhaupt Text gelesen wird.

### Verifikation
- `cargo test` → `base64_matches_known_vectors` (RFC 4648) **und**
  `base64_matches_the_system_on_a_real_file` (182 618 Bytes, identisch zum System-`base64`).
- `pnpm build` grün; PDF-Worker und Worker-Chunks liegen im Build.

### Office-Formate (gleicher Abend)
- **Word** (`docx`, `odt`) → `mammoth` → HTML → **DOMPurify** (derselbe Säuberer wie Markdown).
- **Tabellen** (`xlsx`, `xls`, `ods`, `csv`) → SheetJS; die Tabelle wird **hier Zelle für
  Zelle gebaut und escaped**, nicht von der Bibliothek als HTML geliefert — sonst müsste man
  ihr zweimal vertrauen. Gedeckelt auf 300 Zeilen × 40 Spalten.
- **`rtf`** → grobe Text-Extraktion (Steuerwörter und Klammern weg). Ehrlich: lesbar, ohne Layout.
- **`doc`/`ppt`/`pptx`/`odp`** → klare Meldung („Dieses Format kann nicht angezeigt werden“)
  statt Buchstabensalat. Alte Binärformate brauchen einen Konverter, den bündeln wir nicht.
- **PDF-Standardfonts** liegen in `public/pdfjs/standard_fonts/` (16 Dateien) und werden über
  `standardFontDataUrl` geladen → die „system font“-Warnung ist weg.

### Offen
- **`xlsx` 0.18.5 ist die letzte npm-Ausgabe (2022)** und hat Advisories zum Parsen *fremder*
  Dateien (Prototype Pollution). Für die Vorschau *eigener* Dateien akzeptiert, aber auf der
  Liste: auf den offiziellen SheetJS-Build wechseln.
- Tabellenvorschau zeigt nur das **erste** Blatt, ohne Formeln und Formatierung.
- PDF, Bilder und Markdown sind von Hand bestätigt; **Office noch nicht durchgeklickt**.

## 20. Morgen-Liste (2026-10-15, spät)

### Was schon persistiert wird und was nicht (geprüft im Code)
| Zustand | gespeichert? | wo |
|---|---|---|
| Panel-Breiten (links/rechts) | **ja** | `localStorage` `ds-ide.ui.v2` |
| Seitenleiste eingeklappt | **ja** | dito |
| Work-Area sichtbar | **ja** | `ds-ide.workarea` |
| Arbeitsverzeichnis | **ja** | `settings`-Tabelle |
| Anzeigename, Modell, Toggles, Sprache | **ja** | `settings`-Tabelle |
| **Fenstergröße/-position** | **nein** | Tauri speichert das nicht von selbst |
| **Aktive Sitzung** | **nein** | nur im Speicher |
| **Offene Work-Area-Tabs** | **nein** | nur im Speicher |
| **Zuletzt geöffnete Datei** | **nein** | nur im Speicher |

Fenster: dafür gibt es `tauri-plugin-window-state` (genau dessen Zweck).
Sitzung/Tabs/Datei: gehören in die `settings`-Tabelle bzw. in eine kleine
`ui_state`-Ablage — vier Felder, kein großes Vorhaben.

### Offene Punkte
1. **`pptx`** zeigt bewusst „Dieses Format kann nicht angezeigt werden". Präsentationen
   sind ein eigener Fall (keine brauchbare Browser-Bibliothek für echtes Layout);
   entweder eine einfache Folien-Extraktion oder ehrlich draußen lassen.
2. **`doc`/`ppt` (Binärformate)** bleiben draußen — brauchen einen Konverter.
3. **`xlsx` 0.18.5** gegen den offiziellen SheetJS-Build tauschen (Advisories).
4. **Companion-System**: die Alternative zum Mavis-Partner.
   - Rolle **Partner**, nicht Agent — im Recall und in der Persona sichtbar.
   - Wer Seekys Gedächtnis ändert, wird markiert: `origin` (`seeky` | `user`) und
     `edited_at` in `memories`; im Recall steht dann „vom Nutzer geändert".
   - Löschen als **Stilllegen**, nicht physisch entfernen.
   - Gedächtnis-Einsicht wandert dafür aus dem Work-Area-Tab nach
     Einstellungen → „Konto & Modell" bzw. „Personalisierung".

→ Punkte 1–4 sind in Abschnitt 21 abgearbeitet.

## 21. Persistenz, Vorschauen, Gedächtnis (2026-10-16)

Fünf Commits, einer pro Punkt, in der Reihenfolge der Liste aus §20: `05a5165`
(Fenster), `de8b120` (UI-Zustand), `b5b9552` (Folien), `0769925` (SheetJS),
`92bfc5f` (Gedächtnis).

### Was jetzt persistiert wird (Stand nach dieser Runde)
| Zustand | gespeichert? | wo |
|---|---|---|
| Panel-Breiten (links/rechts), Seitenleiste eingeklappt | **ja** | `localStorage` `ds-ide.ui.v2` |
| Work-Area sichtbar, offene Tabs, aktiver Tab, zuletzt geöffnete Datei | **ja** | `settings`-Tabelle, Schlüssel `ui.workarea` |
| Aktive Sitzung | **ja** | `settings`-Tabelle, Schlüssel `ui.activeSession` |
| Fenstergröße/-position/maximiert | **ja** | `.window-state.json` im Konfigverzeichnis |
| Arbeitsverzeichnis, Anzeigename, Modell, Toggles, Sprache | **ja** | `settings`-Tabelle |

- **Fenster**: `tauri-plugin-window-state`, aber mit Absicht nur
  `SIZE | POSITION | MAXIMIZED`. `VISIBLE` fehlt, weil sonst die Fensterlage vor dem
  Splash entscheidet, wann das Hauptfenster erscheint; das Splash-Fenster steht auf
  der Denylist, damit sein Rechteck nicht in die Datei wandert.
- **Sitzung/Tabs/Datei**: `src/lib/uiState.ts` schreibt `ui.*`-Schlüssel, entprellt
  und fehlertolerant — ein gemerkter Tab darf keinen Klick aufhalten. Der alte
  localStorage-Schlüssel `ds-ide.workarea` wird einmal importiert und gelöscht.
  Wiederherstellen repariert statt zu scheitern: unbekannte Tab-Arten, Duplikate
  und ein veralteter aktiver Tab werden verworfen, und der bereinigte Zustand wird
  zurückgeschrieben.

### Verifikation (nicht nur Build)
- Fenster: auf 1100×650 bei 200,180 gezogen, beendet, neu gestartet — kam genau dort
  zurück (X11-Lauf, `xdotool getwindowgeometry`).
- UI-Zustand: eine absichtlich falsche Sitzungs-ID wurde beim Start durch die echte
  neueste ersetzt (also schreibt die App wirklich), der alte localStorage-Schlüssel
  verschwand und wurde als `ui.workarea` in die Datenbank übernommen, und eine
  kaputte Tab-Liste kam als „progress + memory" mit repariertem aktivem Tab zurück.
- `cargo test`: 17 grün, 9 ignoriert. Neu: Stilllegen/Zurückholen-Runde, der
  `origin`-Marker, der Recall-Text und die Migration einer alten `memories`-Tabelle.

### Vorschauen
- **`pptx` ist jetzt lesbar**: `jszip` entpackt, `ppt/slides/slideN.xml` liefert
  Absätze; jede Folie wird eine Karte mit Titel und Text, numerisch sortiert,
  Folien ohne Text fallen weg. Ehrlich Text, kein Layout — es sieht aus wie eine
  Folienliste, weil es eine ist. Geprüft mit einem künstlichen Deck (escapte
  `<script>`-Tags, Emoji, textlose Folie, Folie 10 nach Folie 3) und mit einem
  echten 12-Folien-Canva-Export.
- **`xlsx` läuft auf dem offiziellen SheetJS-Build** (0.20.3 vom cdn, nicht npm).
  `pnpm audit`: vorher 5 Verwundbarkeiten (2 low, 1 moderate, 2 high — beide hoch
  betrafen `xlsx`), nachher 3 (2 low, 1 moderate). Die Tabelle rendert unverändert;
  geprüft mit einem echten Bewertungsraster. Upgrades kommen künftig von
  `cdn.sheetjs.com`, nicht aus der npm-Registry.
- **`doc`/`ppt`/`odp`** bleiben draußen — klare Meldung, kein Buchstabensalat.

### Companion-System
- Die Gedächtnis-Einsicht ist aus dem Work-Area-Tab nach **Einstellungen →
  Personalisierung** gewandert; Tab, Toolbar-Knopf und `MemoryTab.vue` sind weg.
- `memories` hat `origin` (`seeky` | `user`), `edited_at` und `retired_at`.
  Schreibt der Nutzer im Panel einen Eintrag um, wird er markiert; im Recall steht
  dann `(changed by the user)`, und die Persona erklärt, dass das die Worte des
  Nutzers sind. Korrigiert Seeky sich selbst, fällt der Marker wieder weg — er
  hängt am Text, nicht an der Zeile.
- **Nichts wird mehr gelöscht**: Stilllegen setzt `retired_at`, der Eintrag
  verschwindet aus Recall und Liste und liegt im Panel unter „Stillgelegt", von wo
  er zurückgeholt werden kann. Auch der Kurations-Lauf legt nur still.
- Persona und Recall sagen jetzt *Partner* und nennen die Bereiche in Worten
  (`[you]` / `[user]` / `[session]`) statt der Rohwerte aus der Datenbank.
- Geprüft: an der echten Datenbank kamen alte Zeilen als `seeky`-Einträge zurück,
  ein umgeschriebener Eintrag erschien als „von dir geändert" neben „Stillgelegt (1)",
  und die Schreibwege liefen über genau die `invoke`-Aufrufe des Panels.

### Lernnotiz: synthetische Eingaben erreichen diese App nicht
Der Rechner hier ist KDE Plasma auf Wayland. XWayland lässt den Zeiger nicht
bewegen (`xdotool mousemove` bleibt bei 1920,0 stehen), und `xdotool key` kommt
auch nach `windowfocus` nicht in der Webview an. Klicks und Tasten sind von außen
also nicht zu setzen. Konsequenz für Prüfungen: Zustand über die Datenbank setzen
und das Ergebnis im Screenshot ansehen, oder die Aufrufe des Panels aus der App
selbst auslösen (temporär, danach zurückgenommen). Die Knöpfe des Gedächtnis-Panels
sind deshalb *nicht* mit der Maus durchgeklickt — gesagt, damit es niemand für
erledigt hält, der es nicht ist.

### Offene Punkte
1. **`doc`/`ppt`/`odp`** brauchen einen Konverter; `odp` wäre am ehesten machbar
   (ebenfalls ein Zip, `content.xml` mit `draw:page`).
2. Tabellen zeigen nur das **erste** Blatt, ohne Formeln und Formatierung.
3. Zwei Advisory-Reste außerhalb des Themas: `dompurify` in `monaco-editor` (low,
   gepatchte Version wäre ≥ 3.4.16) und ein transitives `sprintf-js` (moderate).
4. Gedächtnis: der Nutzer kann Einträge **bearbeiten und stilllegen, aber noch
   keinen neuen anlegen** — der Weg „Nutzer schreibt eine Notiz" fehlt.
5. Gedächtnis-Einsicht sitzt unter *Personalisierung*; „Konto & Modell" ist
   unberührt geblieben.
6. Folien: reiner Text. Bilder, Notizen und Reihenfolge der Formen bleiben außen vor.

