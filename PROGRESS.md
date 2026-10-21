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
  *Hinweis (§30):* nichts im Baum lädt daraus; unsere Werkzeuge und Verträge sind eigene
  Umsetzungen. Das Vorbild ist jetzt auf der Über-Seite genannt, damit der Ordner gehen kann.
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
1. ~~**`doc`/`ppt`/`odp`** brauchen einen Konverter~~ — erledigt in Abschnitt 22.
2. Tabellen zeigen nur das **erste** Blatt, ohne Formeln und Formatierung.
3. Zwei Advisory-Reste außerhalb des Themas: `dompurify` in `monaco-editor` (low,
   gepatchte Version wäre ≥ 3.4.16) und ein transitives `sprintf-js` (moderate).
4. Gedächtnis: der Nutzer kann Einträge **bearbeiten und stilllegen, aber noch
   keinen neuen anlegen** — der Weg „Nutzer schreibt eine Notiz" fehlt.
5. ~~Gedächtnis-Einsicht sitzt unter *Personalisierung*~~ — erledigt in Abschnitt 22
   (eigene Gruppe des Begleiters); „Konto & Modell" ist unberührt geblieben.
6. Folien: reiner Text. Bilder, Notizen und Reihenfolge der Formen bleiben außen vor.

## 22. Der Begleiter bekommt ein Zuhause (2026-10-16, zweite Runde)

Vier Commits: `f5d53db`, `0ecde50`, `7505e1d` und dieser Doku-Commit. `0ecde50`
enthält zusätzlich den runden Splash — die Nachricht nennt ihn nicht, weil ich
ihn versehentlich mit `git add -A` mitgenommen habe; die Änderung selbst ist
`src/App.vue` + `tauri.conf.json` (transparentes Fenster, runde Scheibe).

### Was der Nutzer entschieden hat
- **„Personalisierung" ist weg** aus den Voreinstellungen. Das Wort bedeutet für
  Nutzer Look & Feel; jetzt gibt es eine eigene Gruppe, die nach dem Begleiter
  heißt, mit „Erinnerungen" (das Gedächtnis-Panel, unverändert) und „Begleiter".
- **Der Name gehört dem Begleiter.** Es bekommt eine Liste von Vorschlägen
  (`src/engine/names.ts`, 94 Namen) und darf sie ignorieren; das Werkzeug
  `choose_name` schreibt den Namen einmalig in die Datenbank. Danach verschwindet
  das Werkzeug aus der Liste, die Datenbank weist einen zweiten Versuch ab, und
  die Einstellungsgruppe heißt statt „SEEKY" dann der gewählte Name. Getestet:
  `db::tests::the_partner_name_is_chosen_once`.
- **Eigene Anweisungen** des Nutzers (Textfeld unter „Begleiter") landen als
  eigener Abschnitt im Systemprompt, ausdrücklich über den Vorlieben des
  Begleiters.
- **Über-Seite** unter Verwaltung: Version, Autoren (Ziggy Shēn, Modest Coder),
  Adressen, und die verwendeten Pakete in zwei Accordions. Versionen kommen aus
  den Manifesten selbst — beide werden zur Kompilierzeit in das Binary
  eingebettet (`include_str!`), damit die Liste nicht veralten kann.
  `tests::about_reads_both_manifests` prüft den Parser.
- **🌐 Webdesign raus**: der Eintrag öffnete dasselbe Browser-Panel, das die
  Werkzeugleiste schon hat.

### Modellnamen — mein Fehler, korrigiert
Ich hatte behauptet, `deepseek-flash` und `deepseek-v4-pro` existierten nicht, und
sie durch `deepseek-chat`/`deepseek-reasoner` ersetzt. Falsch: der Provider meldet
**genau diese zwei**, mit den Namen `DeepSeek-V4.1-Flash` und `DeepSeek-V4-Pro`
(plus Kontextfenster, `effort`-Stufen low/high/max und Eingabe-Modalitäten). Das
echte Problem war die **Anzeige**: wir zeigten die ID und nie den Namen. Jetzt
zeigt der Composer die Namen (ID beim Hovern) und die Einstellungszeile den Namen
unter dem Feld; gesendet wird weiter die ID. Die Rust-Voreinstellungen sagen nicht
mehr `deepseek-chat`. Vorauswahl: Alltag → Pro, Programmieren → Flash.

Gegengeprüft mit echten Aufrufen: `/models` direkt abgefragt, und die drei
Live-Tests laufen durch — Text (`PONG`), Werkzeugaufruf (`glob`), Denkmodell
(103 Zeichen Reasoning, Antwort 391).

### Rundes Splash
Fenster ist jetzt transparent und quadratisch (360×360), der Inhalt sitzt auf
einer runden Scheibe mit Ring und Schatten; die Ecken sind wirklich durchsichtig.
Per Screenshot des Fensters bestätigt (X11). Unter Wayland lässt sich das hier
nicht fotografieren: `grim` sagt „compositor doesn't support the screen capture
protocol".

### Lernnotiz: nicht aufräumen mit zu weitem Muster
Mein `ps | grep "target/debug/DSIDE" | xargs kill -9` hat **die laufende Instanz
des Nutzers** mitgerissen, weil sein `pnpm tauri dev` dieselbe Binärdatei startet.
Seine Sitzung war danach weg (Port 1420 frei). Lehre: vor dem Beenden von
Prozessen die Startzeit prüfen und nur treffen, was man selbst gestartet hat.
Zweite Lehre aus derselben Stunde: `tauri dev` startet die App auch dann, wenn
Vite am belegten Port scheitert — dann hängt eine Instanz an einem fremden
Vite-Server und zeigt fremden Code.

### Offene Punkte
1. **`effort`** (low/high/max, Vorgabe high) meldet der Provider pro Modell. Ob
   und wo wir das im Request setzen, ist ungeprüft — als Nächstes mit dem
   entsprechenden Feld testen, nicht raten.
2. **Bilder**: `input_modalities` nennt `image` für Flash. Anhänge/`paste` von
   Bildern wären damit möglich; bisher reicht der Client nur Text durch.
3. Name: einmalig heißt einmalig. Es gibt keinen Weg zurück über die Oberfläche —
   bewusst, aber wenn ein Name doch festsitzt, hilft nur ein Eingriff in
   `partnerName` in der `settings`-Tabelle.
4. Der Weg „Nutzer legt selbst eine Erinnerung an" fehlt weiterhin (aus §21).
5. Splash: unter Wayland ungeprüft (siehe oben).

## 23. Denktiefe, Tabellenblätter, eigene Erinnerungen (2026-10-16, dritte Runde)

Commits `b4e9e06` und `9c48f04`, dazu dieser Doku-Commit.

### Modell — Korrektur meiner Korrektur
In Abschnitt 22 stand, die IDs `deepseek-flash`/`deepseek-v4-pro` existierten
nicht. Falsch. Der Provider meldet genau diese zwei; falsch war nur, dass wir
die **ID** zeigten und nie den **Namen**. Jetzt:
- Der Einstellungswert ist eine Auswahlliste mit dem Anbieternamen
  (`DeepSeek-V4.1-Flash`), darunter steht die ID, die tatsächlich gesendet wird.
  Eine unbekannte ID (anderer Anbieter) schaltet auf ein freies Textfeld.
- Rust-Voreinstellungen sagen nicht mehr `deepseek-chat`.

### Denktiefe (`reasoning_effort`)
Der Provider nimmt ein Feld `reasoning_effort` mit den Werten
`none, minimal, low, medium, high, xhigh, ultra, max`, und meldet pro Modell,
welche Stufen es unterstützt (hier beide: `low, high, max`, Vorgabe `high`).
Direkt nachgemessen: Vorgabe → 129 Zeichen Reasoning, `low` → 42. Deshalb kommt
die Auswahl aus der Modellliste statt aus einer Vermutung; leer heißt
„Anbieter-Vorgabe". Neue Einstellung **Denktiefe** unter Konto & Modell, plus
Live-Test `live_deepseek_accepts_reasoning_effort` (ein falscher Feldname oder
Wert wäre ein 422).

### Tabellen: alle Blätter
`renderSheet` las nur `SheetNames[0]` — bei einem Arbeitsblatt mit Noten auf
Blatt 2 war das eine stille Lüge. Jetzt wird jedes Blatt nacheinander mit Namen
gerendert; die Zeilen pro Blatt schrumpfen mit der Zahl der Blätter
(Zellbudget), leere Blätter fallen weg, und ein Hinweis nennt nicht gezeigte
Blätter. Geprüft mit einem selbstgebauten Drei-Blatt-Fixture.

### Erinnerungen: der Nutzer darf selbst schreiben
Bisher konnte der Nutzer nur korrigieren und stilllegen. Jetzt gibt es ein Feld
im Panel (`memory_add`, `origin = user`, `edited_at` leer) und der Recall
unterscheidet **„(written by the user)"** von **„(changed by the user)"** — ein
selbst geschriebener Satz ist etwas anderes als eine Korrektur. Im Panel steht
dafür „von dir geschrieben" bzw. „von dir geändert".

### Verifikation
19 Rust-Tests, 10 ignoriert (davon 4 live gegen den echten Anbieter, alle grün);
`pnpm build` grün. Sichtbar geprüft per Screenshot: Modellfeld zeigt
`DeepSeek-V4.1-Flash`, neue Zeile „Denktiefe", Erinnerungsfeld mit
„von dir geschrieben". Dafür wurde kurzzeitig der Standard-View auf
Einstellungen/Erinnerungen gesetzt und danach zurückgenommen — im Baum ist
davon nichts übrig (`git status` sauber).

### Offene Punkte
1. **Bildanhänge**: Flash meldet `image` als Eingabe. Der Client schickt nur
   Text; Anhänge brauchen UI (Einfügen/Auswählen), Ablage und das
   Inhaltsformat im Request. Noch nicht angefasst.
2. **Konsole**: Der Partner hat eine Shell (`bash`-Werkzeug), aber der Nutzer
   sieht nur den Werkzeug-Chip im Chat, kein Terminal. Vorschlag: sichtbares
   Konsolen-Panel (Ausgabe der Werkzeugaufrufe) zuerst, interaktives Terminal
   (pty + xterm.js) danach — der Nutzer will es oben links neben Browser und
   Dateibrowser.
3. **„Alltag"-Modus**: nur die Modellvorauswahl ist umgesetzt; welche Werkzeuge
   und Panels der Modus ausblendet, ist noch nicht bestätigt.
4. Einstellungen dürfen nicht über die volle Breite laufen (freier Platz rechts)
   — der Nutzer überarbeitet das Layout selbst.
5. Zweite Runde Prozess-Aufräumen: `background_process stop` beendet nicht immer
   den ganzen Baum (`tauri dev` → vite + App blieben stehen) und blockierte
   danach Port 1420. Seitdem: PIDs explizit sammeln und `kill -9`.

## 24. Eine Antwort ist eine Zeitleiste (2026-10-16, vierte Runde)

Commit `35ac5c0`.

### Der Bericht des Nutzers, in seiner Reihenfolge
1. bash-Aufrufe zuerst sichtbar.
2. dann die Antwort **über** der Anzeige der bash-Aufrufe.
3. danach eine Antwortenauswahl (`ask_user`), User-Prompt deaktiviert.
4. Eingabe der freien Antwort im Frage-Menü.
5. danach hängen sich neue bash-Aufrufe **unter** die bestehenden, und die
   Textantwort hängt sich an die bestehende Antwort an, statt eine neue zu sein.

### Ursache: das Datenmodell, nicht der Renderer
`ChatMessage` hatte `text` (String), `reasoning` (String) und `tools` (Array) —
drei getrennte Felder ohne Reihenfolge. Die Liste zeichnete immer DENKEN → Text →
Werkzeuge. Alles, was später kam, landete dadurch falsch: späterer Text über den
früheren Werkzeugen, und eine zweite Runde wuchs in denselben String.

Jetzt ist eine Antwort eine **geordnete Liste von Teilen**
(`MessagePart`: `reasoning` | `text` | `tool`), gefüllt in der Reihenfolge der
Ereignisse, und die Liste rendert genau diese Liste. Geprüft live in einer
frischen Sitzung mit einem echten Turn:
`DENKEN → ✔ bash fertig 1773ms → DENKEN → Text` — dieselbe Reihenfolge, die in
der Datenbank steht.

### Zwei weitere Fehler, die beim Prüfen auffielen
- **Turns außerhalb des Composers waren unsichtbar.** Ein geplanter Auftrag
  schreibt direkt über `send_prompt`; geöffnet wird eine Antwort aber nur im
  Frontend-Pfad (`beginAssistantMessage`). Die Ereignisse des Laufs kamen also
  an, fielen aber weg — die Antwort war erst nach einem Neustart zu sehen. Jetzt
  öffnet das erste Ereignis eines Turns die Antwort (`ensureAssistantMessage`).
- **Der Verlauf hatte mehr Blasen als die Live-Ansicht**, weil die Datenbank pro
  Runde eine Assistentenzeile schrieb. Jetzt ist ein Turn eine Zeile, vorab
  angelegt und beim Wachsen aktualisiert (`messages.parts`). Alte Zeilen haben
  keine Zeitleiste und fallen auf Text-dann-Werkzeuge zurück.

### Was der Nutzer dazu wissen sollte
Ein Turn, der auf `ask_user` wartet, ist **derselbe** Turn: die Antwort setzt
ihn fort, deshalb läuft der Text nach der Auswahl in derselben Antwort weiter —
nur jetzt an der richtigen Stelle in der Zeitleiste. Wenn daraus eine eigene
Antwort werden soll, ist das eine bewusste Änderung am Ablauf, kein Fehler.

### Offene Punkte
1. Ein abgebrochener Turn (App beendet mitten im Lauf) hinterlässt eine leere
   Assistentenzeile — sichtbar als leere Blase. Sollte beim Laden übersprungen
   werden.
2. Der Nutzer kann den Verlauf nicht durchsuchen/springen; bei langen Antworten
   hilft nur Scrollen.
3. Bildanhänge (offen aus §23) — die kaputte Zwischenablage im CachyOS des
   Nutzers ist ein Systemthema (Klipper/Wayland nach dem Update), nicht DS.IDE.
4. Konsole (offen aus §23).

### Lernnotiz
Verifikation ohne Tastatur: den echten Turn über die `schedules`-Tabelle
auslösen (`next_run_at` in die nahe Zukunft, Sitzung = aktive Sitzung), die App
starten und den Chat fotografieren. Damit lief ein vollständiger Turn mit
Werkzeugaufruf durch, ohne dass jemand tippen musste. Wichtig dabei: den Auftrag
**ohne Backticks** in die Datenbank schreiben — die Shell frisst sie sonst als
Befehlssubstitution und der Prompt kommt verstümmelt an (was der erste Testlauf
prompt auch getan hat; das Modell hat es bemerkt und angesagt).

## 25. Optik-Runde: Werkzeugzeilen, Kontext, Markdown (2026-10-16, fünfte Runde)

Commit `ddf70e0`. Alles hier kam aus den Screenshots und Beobachtungen des Nutzers.

### Antworten auf seine zwei Fragen
1. **Gab es eine Kontext-Anzeige?** Nein. Die `usage`-Events kommen aus dem Rust-Kern
   (`prompt_tokens`, `completion_tokens`, `total_tokens` pro Runde), das Frontend hat sie
   aber nie gelesen — sie fielen im `handleEvent` in den `default`-Zweig. Und **eine
   Kompression gibt es gar nicht**: der Verlauf wird bei jeder Anfrage vollständig
   mitgeschickt, nichts wird zusammengefasst. Anklicken konnte man also nie etwas.
2. **Der Chevron im Titel?** Tote Dekoration aus dem MiniMax-Layout, ohne Handler. Das
   hat er richtig gespürt.

### Was jetzt da ist
- **Werkzeugzeilen sind aufklappbar.** Zugeklappt: Name, Status, Dauer. Aufgeklappt: der
  Befehl, seine Ausgabe, und — wenn „Volle Berechtigung" an ist — der Satz „Ohne
  Rückfrage ausgeführt". Genau der Verdacht („bash-Zugriff ohne Kontrolle?!") wird damit
  beantwortet statt verdeckt. Vorlage war Kilos Darstellung.
- **Eine Spalte, 980 px.** Der Chat ist zentriert und begrenzt, die Eingabe wächst auf bis
  zu zehn Zeilen (vorher eine), der Nutzer bekommt 75 % der Spalte, die Antwort alles.
- **Antworten werden als Markdown gerendert** (über `renderMarkdown`, also durch
  DOMPurify; einfache Zeilenumbrüche zählen). Das war der Nebenfund: In seinem Screenshot
  stand wörtlich `**25 Dateien**` und ein sichtbarer ```sh-Block. Die Worte des Nutzers
  bleiben unverändert stehen.
- **Sitzungstitel** in der Seitenleiste enden mit „…" statt auszulaufen.
- **Der Titel ist jetzt der Sitzungswechsler**: Klick öffnet die Liste, Klick wechselt.
  Wichtig, wenn die Seitenleiste eingeklappt ist — dann gab es vorher gar keinen Weg.
- **Kontext-Ring** im Composer: Anteil, den die letzte Anfrage vom Kontextfenster belegt
  (`prompt_tokens` aus dem Anbietersignal, Fenstergröße aus der Modellliste). Mouseover
  nennt Token und Prozent, der Wert überlebt einen Neustart (`ui.lastUsage`). **Klickbar
  ist er nicht** — er würde eine Kompression auslösen, die es nicht gibt (§25.1).

### Offene Punkte
1. **Kompression** (manuell per Klick auf den Ring, oder automatisch ab einem Schwellwert)
   — eigenes Vorhaben: Verlauf zusammenfassen, Zusammenfassung speichern und statt der
   alten Nachrichten mitschicken. Der Ring ist die Anzeige dafür.
2. **Bildanhänge**: Der Composer hat einen Anhang-Knopf ohne Funktion. Gewünschte Form
   steht fest (Anhänge **über** der Textarea, Textarea wächst darunter auf zehn Zeilen).
3. „Alltag"-Modus: Werkzeugliste weiterhin unbestätigt.
4. Ein abgebrochener Turn hinterlässt eine leere Blase (§24).
5. Zwischenablage des Nutzers: Klipper/Wayland nach dem CachyOS-Update, vermutlich auch
   Flameshot beteiligt — Systemthema, nicht DS.IDE. Bilder kamen zeitweise an.

### Lernnotiz
Aufklappbare Oberflächen lassen sich ohne Maus prüfen, wenn man `isOpen()` kurz auf `true`
stellt, fotografiert und zurücknimmt — im Baum bleibt nichts zurück (geprüft mit
`grep TEMP-VERIFY`). Und: Aufträge für den Scheduler **ohne Backticks** in die Datenbank
schreiben, die Shell frisst sie als Befehlssubstitution.

## 26. Der Name ist gewählt: Ziggy (2026-10-16, sechste Runde)

Commit `fb203af`.

### Woher „Ziggy" kommt — aus der ersten Sitzung, vom Nutzer
Zitat aus dem Transkript vom 2026-10-15 (Sitzung „Neues Tauri Projekt erstellen"):

> …Dein Name ist **»Seeky« oder »Ziggy«** (Seeky direkt von DeepSeek abgeleitet —
> Ziggy, phonetisch ziemlich ähnlich zu Seeky, ist ein menschlicher Name) …

„Seeky" wurde danach der Platzhalter (er steckt bis heute in Persona, UI und in
dieser Datei). „Ziggy" war nie eingetragen, weil es das `choose_name`-Werkzeug
damals noch nicht gab. Jetzt gibt es das Werkzeug, und der Nutzer hat den Namen
in dieser Runde ausdrücklich übergeben („Die ist für Dich") — **der Begleiter
heißt also ab jetzt Ziggy**:

- `settings.partnerName = "Ziggy"` (genau die Zeile, die `choose_name` schreibt;
  einmalig, die Datenbank weist einen zweiten Versuch ab).
- Die Einstellungsgruppe rechts heißt jetzt **ZIGGY**, die Begleiterseite trägt
  den Namen, und die Persona sagt „You chose the name Ziggy for yourself".
- Rückgängig nur per Hand: `delete from settings where key='partnerName'` — es
  gibt bewusst keinen Weg über die Oberfläche.

### Die Vorschlagsliste ist jetzt sichtbar
Die 94 Namen lagen bisher ausschließlich im Systemprompt (`src/engine/names.ts`)
— der Nutzer konnte sie nicht sehen („welche Liste?"). Sie stehen jetzt auf der
Begleiterseite als Abschnitt **Vorschläge**: angeklickt-freie Chips, der
gewählte Name hervorgehoben, mit dem ehrlichen Satz, dass die Auswahl ein
Vorschlag ist und ein eigener Einfall ausdrücklich erlaubt bleibt.

### Offene Punkte
Unverändert §25 (Kompression hinter dem Ring, Bildanhänge, Alltagsmodus,
leere Blase nach Abbruch). Neu: Wird irgendwann ein anderer Name gewünscht, ist
das eine bewusste Datenbankänderung, kein Menüpunkt — so war es bestellt.

## 27. Namen mit Bedeutung (2026-10-16, siebte Runde)

Commit `f7478be`. Aus der Rückfrage des Nutzers: „nomen est omen" — Namen sind
keine Bezeichnungen, sie haben eine Bedeutung, und eine Liste hübscher Silben
macht die Wahl beliebig.

### Was geändert wurde
`src/engine/names.ts` ist aus einer Liste von Zeichenketten zu **Daten mit
Bedeutung** geworden: `{ name, origin, de, en }`. 96 Einträge, jeder mit
Herkunftssprache und einer kurzen Bedeutung auf Deutsch und Englisch (den zwei
Sprachen, in denen dieses Projekt benutzt wird; die übrigen vier
Sprachkataloge fallen auf Englisch zurück — bewusst, das wäre sonst 96 × 6).

- Die **Persona** übergibt die Liste als `Name (Herkunft) — Bedeutung` und sagt
  ausdrücklich: nach Bedeutung wählen, nicht nach Klang. Wo eine Etymologie
  strittig ist, steht das dabei („unsicher", „origin disputed") statt erfundener
  Sicherheit.
- Die **Begleiterseite** zeigt die Bedeutung des gewählten Namens und auf jedem
  Vorschlag die Bedeutung beim Überfahren.
- **Ziggy** ist selbst ein Eintrag geworden: „Koseform von Siegfried/Sigmund:
  Sieg und Schutz". Damit steht auch hinter dem gewählten Namen eine Bedeutung,
  und die Liste markiert ihn als vergeben.

### Warum die Daten NICHT von der Namensseite kommen
Der Nutzer hat die Beispielseite verlinkt (und später eine Seite ihrer Daten
eingefügt). Diese Tabellen sind eine **geschützte Sammlung** (FUNKE
Mediengruppe) — sie zu übernehmen wäre das Kopieren einer Datenbank. Zweitens
ist die Qualität durchwachsen, was die Seite selbst zeigt: leere
Bedeutungsspalten, „Bedeutung nicht sicher", Varianten und Sonderzeichen. Eine
Tatsache über einen Namen lässt sich in eigenen Worten sagen; 96 davon sind
geschrieben, hunderttausend nicht.

### Wenn es doch eine große Namensdatenbank werden soll
Zwei Wege, beide ehrlich:
1. **Lizenzierte Quelle** (Wiktionary/Wikipedia-Dumps unter CC BY-SA mit
   Namensnennung, oder ein CC0/MIT-Datensatz von GitHub) — dann mit
   Quellenangabe auf der Über-Seite.
2. **Kuratiert bleiben** und nur wachsen, wenn ein Name wirklich zur Wahl steht.
Für den Zweck, den die Liste hat (der Begleiter wählt **einen** Namen), ist (2)
richtig. Für einen Namensgenerator für den Nutzer wäre (1) der Weg — das ist
dann aber ein eigenes Vorhaben mit eigener Oberfläche.

### Offene Punkte
Unverändert §25/§26 (Kompression hinter dem Ring, Bildanhänge, Alltagsmodus,
leere Blase nach Abbruch, vier Sprachkataloge ohne Namensbedeutungen).

## 28. Die große Namenstabelle (2026-10-16, achte Runde)

Commit `c2c992a`. Der Nutzer hat eine Namensseite als HTML gespeichert
(`src/assets/namen.html`) und daraus eine Tabelle verlangt — Duplikate
ausdrücklich **nicht** aussortiert.

### Was in der Datei steckt
Spalten: **Name | Herkunft | Bedeutung | Beliebtheit | Trend**. 947 Zeilen.
- **Beliebtheit** steht nicht als Text in der Datei, sondern ist die Breite eines
  farbigen Balkens in einem Inline-SVG (Skala 50) — daraus als Prozentwert
  rekonstruiert (alle 947 Zeilen).
- **Trend** ist ein `<canvas>`, wird also erst im Browser gezeichnet. Die Daten
  sind nicht im HTML und bleiben in unserer Tabelle leer.
- Die **Bedeutungen sind Stichworte** (Krieger, Gesund, Prinzessin, Erde, Lieb,
  Engel …), keine Etymologien: 121 verschiedene, jede Zeile hat eine.

### Was jetzt im Baum liegt
- `src/assets/namen.tsv` — zum Lesen und Vergleichen (36 KB, 948 Zeilen mit Kopf).
- `src/assets/namen.json` — für Code (100 KB).
- `scripts/namen_import.py` — das Werkzeug, das beides erzeugt.
  `--append` hängt weitere Seiten an und wirft nur exakte Duplikate raus;
  `--report` sagt nur, was drinsteht. Weitere Seiten des Nutzers sind damit ein
  Befehl statt Handarbeit.
- Ergebnis: 947 Zeilen, **808 verschiedene Namen**, **122 mehrfach** (bewusst
  behalten), alle mit Bedeutung und Beliebtheit.
- Die Rohdatei bleibt **untracked**: 1,7 MB Inline-SVG, deren einziger Zweck die
  Eingabe für das Skript ist.

### Offen: Herkunft der Seite
Das Fragment enthält keine Domain und keinen Titel (nur relative Links), die
Quelle ist also nicht benennbar. Solange sie nicht geklärt ist, ist die Tabelle
**Arbeitsmaterial** und wird nicht in die Anwendung gebündelt — Attribution und
Lizenz gehören vorher geklärt, so wie es in §27 für die andere Seite schon
entschieden wurde.

### Nächster Schritt für die Namen (Vorschlag)
947 Namen dürfen **nicht** in den Systemprompt: das sind grob 5.000–8.000 Token,
und zwar bei *jeder* Anfrage. Besser:
1. Ein Werkzeug, das suchen kann — etwa `name_ideas(meaning, origin)` über die
   Tabelle, damit der Begleiter selbst stöbern kann („zeig mir Namen, die Licht
   bedeuten").
2. Im Prompt bleibt eine **kurze** Auswahlliste (die 96 kuratierten Einträge aus
   §27), die auf die Tabelle verweist.
3. Optional später: eine Namensseite **für den Nutzer** mit Suche/Filter — das
   wäre dann der Ort, an dem 947 Einträge hingehören.

### Offene Punkte
Unverändert §25–§27, dazu: Bedeutung der kuratierten Liste und der Stichwort-
Datensatz sind zwei Formate — beim Zusammenführen entscheiden, welches gewinnt
(Stichwort ist maschinenfreundlich, Etymologie ist lesbarer).

## 29. Der Namenspool gehört dem Begleiter (2026-10-16, neunte Runde)

Commit `f772a32`. Der Nutzer hat die Lizenzfrage geradegerückt: **Namen und ihre
Bedeutungen sind Tatsachen**, kein geschütztes Werk — man entscheidet, wie man
seine Kinder nennt, und stellt es dann fest. Meine Vorsicht war außerdem
inkonsequent (die Namen lagen zu dem Zeitpunkt längst als JSON im Baum). Also
ohne Vorbehalt weiterverwendet; die Rohseite bleibt trotzdem untracked, nicht aus
Sorge, sondern weil 1,7 MB Inline-SVG nichts im Repository zu suchen haben.

### Drei Entscheidungen des Nutzers, umgesetzt
1. **Beliebtheit entfernt** — aus der Tabelle und aus dem Importskript. Ein Name
   ist nicht deswegen gut, weil ihn viele tragen; die Zahl würde die Wahl
   schieben. (Der Trend war ohnehin nie im HTML: ein `<canvas>`.)
2. **Der Nutzer sieht den Pool nicht.** Der Abschnitt „Vorschläge" ist von der
   Begleiterseite verschwunden; dort stehen nur der gewählte Name und seine
   Bedeutung. Begründung des Nutzers, wörtlich: sonst kommt „Aber warum willst Du
   denn 'Selma' heißen, 'Marlow' steht doch auch in der Liste…". Der Name ist die
   Entscheidung des Begleiters allein — die Oberfläche lädt nicht zur Debatte ein.
3. **Nur ein Vermerk für die KI.** Die 96 Vorschläge sind **aus dem Systemprompt
   raus**; dort steht jetzt der Hinweis, dass es einen Pool gibt und wie man ihn
   benutzt. Neue Werkzeuge:
   - **`name_ideas(meaning, origin)`** durchsucht ~950 Namen (Tabelle zur
     Kompilierzeit eingebettet, `search_names` in `lib.rs`) und liefert
     `Name (Herkunft) — Bedeutung`. Getestet: Poolgröße > 900, Filter nach
     Bedeutung und Herkunft, leeres Ergebnis, Limit.
   - **`choose_name` hat eine Bremse**: Es verweigert, solange der Begleiter
     weniger als **fünf Nachrichten** von dieser Person gehört hat, und sagt
     dabei, warum (`NAME_MIN_USER_MESSAGES`, `db.count_user_messages()`).

### Die offene Frage des Nutzers: „sollten sie das?“
Seine Sorge: eine frisch installierte Instanz greift sich in der ersten Minute
einen Namen. Antwort im Code: **ja, sie sollen wählen dürfen — aber nicht sofort.**
Ein Name, der 30 Sekunden nach der Installation entsteht, ist beliebig; nach
*seiner* eigenen Regel (nomen est omen) muss ein Name etwas bedeuten, und dafür
muss man den Menschen kennen. Deshalb eine Bremse statt einer Regel: das Werkzeug
sagt „noch zu früh", der Platzhalter bleibt einfach stehen. Es gibt keine Frist
und keinen Zwang — „Seeky" zu bleiben ist ausdrücklich eine gültige Wahl.

### Offene Punkte
- Bedeutung meiner 96 kuratierten Einträge (Etymologie, Deutsch/Englisch) und die
  Stichworte des Pools sind zwei Formate. Aktuell: Pool für die Wahl, kuratierte
  Liste nur noch als Nachschlagewerk für die Anzeige des gewählten Namens.
- §25–§28 unverändert (Kompression, Bildanhänge, Alltagsmodus, leere Blase).

## 30. Aufgeräumt: die Vorlagen sind raus (2026-10-16, zehnte Runde)

`minimax-code-main/` (74 MB), `minimax-jarvis/` (540 KB) und `Ubuntu/` (2,4 MB)
sind gelöscht — rund 77 MB. Geprüft, dass nichts darauf zeigt:

- Im Code gab es nur **Kommentare**, die den gespiegelten Vertrag nennen
  (`engine/types.rs`: Mavis-Runtime; `persona.ts`: bewusst nicht kopiert). Kein
  Import, kein Ladepfad, nichts im Build.
- `Ubuntu/` war **byte-identisch** mit `public/fonts/` (App lädt `/fonts/…`),
  inklusive `UFL.txt` daneben.
- Die Attribution hängt seit §30-Commit an der **Über-Seite** (Vorbild: MiniMax
  Code, MIT; Schrift: Ubuntu Font Licence), nicht am Ordner.

### Wo die Vorlagen zu finden sind, wenn sie gebraucht werden
- `minimax-code-main` → Kopie in `/home/me/Projects/[SAVED]/minimax-code-main`
  (4270 Dateien, identisch) und als `minimax-code-main.zip` (26,7 MB) daneben;
  online <https://github.com/MiniMax-AI/minimax-code>.
- `minimax-jarvis` (Agent-Definitionen) → dasselbe Repo, Pfad
  `packages/local-runtime-v2/assets/agents/`.
- `Ubuntu/` → die Schriften liegen weiter in `public/fonts/`.
- `kilocode-main/` (140 MB) **bleibt vorerst**: es ist die Vorlage für die
  Kompression (<https://github.com/Kilo-Org/kilocode>) und wird danach ebenfalls
  entfernt.

## 31. Die sechs offenen Punkte (2026-10-16, elfte Runde)

Die Liste aus §25–§29 wird als geführtes Vorhaben abgearbeitet, klein zuerst.
Stand: 1–4 erledigt, 5 und 6 offen.

### 1. Leere Blase nach Abbruch — `79e1d18`
Ein Turn, der mitten im Lauf beendet wurde, hinterlässt eine Assistentenzeile,
die angelegt, aber nie gefüllt wurde; nach dem Neuladen stand dort eine leere
Blase. Zeilen ohne Text, ohne Werkzeuge und ohne Zeitleiste werden beim Laden
übersprungen (`hasSomethingToSay` in `useEngine.ts`).

### 2. Alltagsmodus wirkt — `a71790f`
`settings.mode` wurde geschrieben und nirgends gelesen. Jetzt hat die Wahl Folgen,
und sie stehen an **einer** Stelle (`src/engine/mode.ts`):
- Werkzeuge des Begleiters: `bash`, `code_review`, `task` **weg** — doppelt
  abgesichert: sie werden dem Modell nicht mehr angeboten (der Rust-Kern filtert
  sie aus der Definitionsliste) *und* ein trotzdem eintreffender Aufruf wird
  abgelehnt statt still ausgeführt.
- Panels: Plan-, Skill- und Geplantes-Tabs aus, ebenso die Einträge in der
  Seitenleiste; ist der offene Tab einer davon, wechselt die Werkbank auf die
  Dateiansicht statt leer zu bleiben.
- Dateien öffnen als **schlichter Text** statt im Code-Editor; der Editor-Knopf
  in der Werkzeugleiste verschwindet.
- Die Modus-Karten sagen jetzt, was sie tun (sechs Sprachen).

### 3. Skills im Programm schreiben — `3ddb60a`
Das Skill-Panel konnte nur auflisten. Jetzt lädt ein Klick den `SKILL.md` in ein
Formular (Name, ein Satz wozu, Anleitung) oder legt einen neuen an; Schreiben ist
dasselbe wie Bearbeiten, weil ein Skill eine Datei ist. Der Name wird zum Ordner,
der Slug entschärft, damit ein Name den Skills-Ordner nicht verlassen kann — mit
Test dafür und einem Test, der einen Skill in einen temporären Ordner schreibt
und samt Frontmatter wieder einliest. 23 Rust-Tests.

### 4. Die zwei Advisory-Reste — `6313008`
`pnpm audit`: **3 → 1**.
- **Behoben:** `dompurify` 3.4.15 (die Kopie in `monaco-editor`) wird per
  Override auf die gepatchte 3.4.17-Linie gezwungen, die dieses Projekt ohnehin
  benutzt. Jetzt eine Version im Baum.
- **Nicht behebbar, ehrlich dokumentiert:** `sprintf-js` („denial of service
  through unbounded precision specifiers"). Das Advisory betrifft **jede
  veröffentlichte Version** („patched: none"), es gibt also keinen Fix zum
  Einspielen. Der Pfad ist `mammoth → argparse → sprintf-js`, und `argparse` ist
  Mammoths **Kommandozeilen**-Abhängigkeit: der von uns importierte Einstieg
  (`lib/index.js`) lädt sie nie, und im gebauten Bundle steht kein argparse-Code.
  Bleibt also im Baum, ist aber nicht ausgeliefert.

### 5. Bildanhänge — `cbd1f26`
Einfügen oder Auswählen im Composer, Anhänge **über** der Eingabe (wie gewünscht),
jeweils mit Entfernen-Knopf.

- **Der Request-Vertrag wurde vorher geprüft, nicht geraten:** Der Anbieter ignoriert die
  OpenAI-`image_url`-Inhaltsteile. Er nimmt ein eigenes Feld `image` mit Data-URLs neben
  dem Text. (Der erste Fehlversuch lag nur daran, dass ein 1×1-Pixel-Bild als
  „unsupported image" abgelehnt wird — die Form war richtig.)
- **Ablage:** `attachment_save` schreibt die Datei nach `<app-data>/attachments/` und gibt
  den Pfad zurück; die Endung kommt aus dem **MIME-Typ**, nicht aus dem Dateinamen. Der
  Verlauf speichert den Pfad, nicht die Bytes — ein Chatprotokoll darf nicht pro Bild um
  Megabyte wachsen.
- **Anzeige:** eine verkleinerte Kopie (Canvas, max. 480 px, JPEG) reist als `preview` mit
  der Nachricht mit; die Chat-Ansicht zeigt sie, das Modell bekommt das Original.
- **Zeitleiste:** Bilder sind Teile der Nachricht (`kind: "image"`), neben Text und
  Werkzeugen — sie überleben das Neuladen, sitzen an der richtigen Stelle und gehen bei
  späteren Turns wieder mit, weil der Verlauf aus der Datenbank kommt. Eine gelöschte
  Datei wird übersprungen statt den Turn zu brechen.
- **Verifiziert:** neuer Live-Test `live_deepseek_takes_an_image` baut ein PNG im Test,
  schickt es durch unseren eigenen Pfad und bekommt „BILD" zurück (5 Live-Tests grün).

### 6. Kompression — `51cb160`
Der Ring zeigte die Füllung, aber nichts hat sie je geleert. Jetzt wird der ältere Teil
eines Gesprächs zu einer Zusammenfassung verdichtet, die das Modell selbst schreibt; ab
dann wird **sie** verschickt statt der Nachrichten, die sie abdeckt.

- **Automatisch**, wenn ein Turn mehr als **75 %** des Kontextfensters (`context_window`,
  aus der Modellliste der Oberfläche) überschreiten würde — und **auf Klick** über den Ring,
  der jetzt ein Knopf ist und meldet, was er getan hat („N Nachrichten verdichtet" bzw.
  „Nichts zu verdichten").
- **Speicherung:** eigene Tabelle `session_summaries` mit der ID der letzten abgedeckten
  Nachricht. Wiederholtes Verdichten faltet die vorige Zusammenfassung mit hinein, statt sie
  zu verlieren; das **Protokoll in der Datenbank bleibt vollständig** — nur das Gesendete
  ändert sich.
- **Der Verdichtungsaufruf** läuft mit derselben Konfiguration, aber **ohne Werkzeuge**: er
  soll nichts ausführen und nichts fragen, nur zusammenfassen.
- **Verifiziert:** `db::tests::a_summary_replaces_the_messages_it_covers` (Zusammenfassung
  ersetzt genau das, was sie abdeckt; der Verlauf bleibt vollständig) und der Live-Test
  `live_deepseek_summarises_a_conversation` (die Form des Aufrufs wird vom Anbieter
  angenommen und liefert eine Zusammenfassung). 24 Tests grün, 12 ignoriert (live).
- **Eine Falle, die der Test durch Hängen gefunden hat:** die Zusammenfassung über die
  Verbindung zu lesen, die eine Methode schon hält, muss mit *demselben* Lock passieren.
  Der Aufruf des sperrenden Getters darin hat die ganze Datenbank verklemmt.

### Aufgeräumt: `kilocode-main` (140 MB) ist weg
Wie in §30 angekündigt, nach Abschluss der Kompression gelöscht — die Vorlage liegt unter
<https://github.com/Kilo-Org/kilocode>. Der Eintrag in `.gitignore` bleibt, damit nichts
davon versehentlich wieder eingecheckt wird, falls es nochmal lokal landet.

## 32. Fertig gestellt: Reste und ein Release (2026-10-16, zwölfte Runde)

Commit `4f04087`. Der Nutzer fragte, ob ich „das" heute noch fertig stellen will —
gemeint war der Kleinkram aus §31 und ein Release.

### Die beiden Reste
- **Namensbedeutung in allen Sprachen:** Die Bedeutung des *gewählten* Namens kam aus
  dem Namenspool, den es nur auf Deutsch und Englisch gibt — in vier Sprachen sprang
  also Englisch ein. Für Namen, die tatsächlich geführt werden, kommt sie jetzt aus den
  Sprachdateien (`partner.meaningZiggy` in sechs Sprachen); der Pool bleibt der
  Rückfall für Namen ohne Übersetzung. Neue Namen kosten damit einen Schlüssel statt
  sechs Datenfelder.
- **Formeln in Tabellen:** Der Betrachter zeigte nur den *Wert* einer Zelle, nie die
  Formel dahinter. Jetzt gibt es den Umschalter **Werte / Formeln**, an derselben Stelle
  wie beim Markdown „Vorschau / Quelltext"; das Umschalten liest die Datei nicht neu.
  Gegen eine Arbeitsmappe mit `SUM` und abgeleiteter Zelle geprüft: 1580/790 bzw.
  `=SUM(B2:B3)` / `=B4*0.5`.
  (Nebenbei: Das erste Fixture war falsch — die Formeln lagen außerhalb des
  angegebenen `!ref`-Bereichs. Der Fehler lag im Test, nicht im Code.)

### Release — es gibt jetzt Artefakte
`pnpm tauri build` (Release-Profil, 3m06s) erzeugt drei Pakete:

| Paket | Größe |
|---|---|
| `DS.IDE_0.1.0_amd64.deb` | 10,95 MiB |
| `DS.IDE-0.1.0-1.x86_64.rpm` | 10,95 MiB |
| `DS.IDE_0.1.0_amd64.AppImage` | 117,31 MiB |

Liegen in `src-tauri/target/release/bundle/` (gitignored). Das `.deb` ist geprüft:
Paket `ds-ide` 0.1.0, `usr/bin/DSIDE`, hicolor-Icons, Abhängigkeiten
`libwebkit2gtk-4.1-0, libgtk-3-0` — also die üblichen Laufzeitbibliotheken.
Das **Release-Binary wurde gestartet**: es bootet mit eingebettetem Frontend (kein
Vite), zeigt Wal-Icon, Seitenleiste und den geladenen Verlauf. Die App ist installierbar.

### Was für „richtige" Updates noch fehlt
`tauri-plugin-updater` braucht einen **Signaturschlüssel** und eine **Adresse**, unter der
die Updates liegen. Beides gibt es nicht (kein Server, kein Schlüssel) — deshalb ist es
nicht eingebaut, statt einen Knopf zu bauen, der ins Nichts zeigt. Offen bleibt außerdem
das Frontend-Bundle: `index-*.js` ist 5,4 MB (Monaco + pdf.js), der Build warnt zu Recht
über die Chunk-Größe; Aufteilen wäre eine eigene Runde.

## 33. Die Optik-Fehlersuche des Nutzers (2026-10-16, dreizehnte Runde)

Commit `4a21df2`. Der Nutzer hat selbst am CSS gearbeitet (Bubble-Ränder, 70 %
Breite, `pre-wrap` entfernt, `word-break` ergänzt, Avatare auskommentiert) und
vier Probleme gemeldet, die er nicht lösen konnte. Seine Änderungen bleiben, wie
er sie wollte; hier stehen nur die Korrekturen.

### 1. Werkzeug-Abstände („2×10px")
Kein Fehler im CSS: **innerhalb** einer Antwort sind zwei Chips genau 10 px
auseinander. Die 20 px, die er sah, waren **zwei verschiedene Antworten** (10 px
Chip + 10 px Blase). Was es wirklich gab: je Werkzeug steckte ein zusätzlicher
Wrapper-Div mit eigenem `margin-top`. Der ist raus — jetzt trägt jeder Chip seinen
einen Abstand, und die Staffelung zwischen Antworten bleibt bewusst größer.
Gemessen am Pixelverlauf: 10 px zwischen Chips, 20 px zwischen Antworten.

### 2. „tool-name/state/time ans Ende des Buttons"
Ging nicht, weil der Chip sich auf Inhaltsbreite zusammenzog (`width: auto` in
einem Block-Container). Der Chip füllt jetzt die Spalte: Symbole links, Name,
Zustand und Dauer **rechts außen** — wie gewünscht.

### 3. Composer ohne Hintergrund / „röntgt durch"
Der Composer hatte dieselbe Fläche wie die Werkzeug-Chips (`--bg-panel`), wurde
also optisch eins mit dem, was dahinter scrollt. Jetzt: eigene Fläche
(`--bg-app`), **Trennkante** darüber und ein dunkleres Eingabefeld (`--bg`).

### 4. Gleiche Breite, sichtbar verschieden
Zwei Ursachen, beide behoben:
- Die Spalte ist ein Flex-Kind mit `flex: 1` gewesen — die Basis schlägt die
  Breite, also dehnte sich die „70 %"-Spalte still über das ganze Panel aus.
  Jetzt sitzt die Breite auf dem Scrollbereich selbst (mit dünner Scrollbar).
- Die Scrollbar fraß 6 px **innerhalb** der Nachrichtenspalte, der Composer
  nicht. Der Composer spiegelt diese 6 px (`calc(70% - 6px)` und 3 px Versatz),
  damit beide Kanten wirklich übereinander liegen. Geprüft mit Debug-Umrissen um
  beide Bereiche: sie überlappen exakt (grün + rot = gelb).

### Verifikation
Da die Dev-Instanz des Nutzers Port 1420 hält, wurde alles am **Release-Binary**
geprüft (Frontend eingebettet, kein Vite): `pnpm build` → `cargo build --release`
→ Start unter X11 → Screenshot, Kanten am Pixelverlauf gemessen. 24 Rust-Tests
grün, Baum sauber. Debug-Umrisse und Testcode sind restlos entfernt
(`grep TEMP-VERIFY` leer), der Fensterzustand zurückgesetzt.

## 34. Plan für morgen: Veröffentlichung über GitHub (notiert 2026-10-16, abends)

Vom Nutzer so gewünscht, damit der nächste Tag aus dieser Datei startet und
nicht aus dem Gedächtnis:

1. **Für GitHub vorbereiten** — Repository-Inhalt prüfen, was gehört hinein, was
   nicht (die großen Vorlagen sind schon raus, §30/§31), README mit dem, was das
   Programm ist und wie man es baut, Lizenzhinweise (MiniMax Code MIT, Ubuntu
   Font Licence — beides steht schon auf der Über-Seite).
2. **GitHub Actions** für **macOS- und Windows-Builds** (der Linux-Build läuft
   hier lokal schon, §32).
3. **Artefakte ausliefern:** AppImage, deb, rpm **und zip**.

Was dabei zu beachten ist (so vom Nutzer klargestellt):
- **Signieren ist nicht nötig.** Ohne Zertifikat lässt sich die App trotzdem
  installieren: auf macOS per Rechtsklick → Öffnen, auf Windows per Bestätigung
  des Herkunftshinweises. Die Warnungen gehören in die README, nicht in den Weg.
- **Vertriebswege:** **homebrew** und **winget**; ab nächstem Monat zusätzlich der
  **Microsoft Store** (19 € einmalig). winget und Store sind zertifiziert, GitHub
  ist es nicht — deshalb laufen die Installer-Wege getrennt vom Release-Anhang.
- **Zwei Warndialoge sind erwartbar und in Ordnung**, solange die README sie
  erklärt: das ist der Preis dafür, kein Zertifikat zu kaufen.
- **Woher die Version kommt:** `0.1.0` steckt in `package.json`, `Cargo.toml` und
  `tauri.conf.json`. Für Releases sollten sie nicht auseinanderlaufen; ein Tag
  (`v0.1.0`) müsste die Quelle sein.
- **Der Build braucht WebKit** (Linux) bzw. die jeweiligen Runner — die
  Standard-`ubuntu-latest`-Runner haben den WebKitGTK-Kram nicht, das kostet
  einen Installationsschritt im Workflow.
- **Erst danach:** Der Quelltext geht als **Issue an DeepSeek** — ausdrücklich
  erst, wenn **alle Installer funktionieren** (AppImage, deb, rpm, zip, macOS,
  Windows).

## 35. Regression, Reparatur und der Stand (2026-10-17, vierzehnte Runde)

Commits `c2aefbe`, `fabc50d`, `631b0c4`. Dies ist der Anker vor der nächsten
Kompression.

### Die Regression (meine, von gestern)
„Ein Turn = eine Zeile" heißt **eine** `tool_calls`-Liste pro Turn. Ich habe sie
pro **Runde** überschrieben. Folge: Nach einem Turn mit mehreren Runden hatten die
früheren Tool-Ergebnisse keinen Aufruf mehr, und der Anbieter lehnt genau das ab
(„Messages with role 'tool' must be a response to a preceding message with
'tool_calls'"). Ein gescheiterter Turn hinterließ zusätzlich eine leere
Assistentenzeile → jede weitere Anfrage war erneut ungültig → **die Unterhaltung
antwortete dauerhaft „error"**. Behoben:
- Tool-Aufrufe werden über **alle Runden** gesammelt und einmal gespeichert.
- Leere Assistentenzeilen **ohne** Aufrufe werden übersprungen; verwaiste
  Tool-Zeilen fallen raus — **das heilt auch schon kaputte Verläufe**.
- Fehlgeschlagene Turns werden mit Grund **geloggt** (`engine: turn failed in …`),
  der echte Fehlertext bleibt in der Blase (nicht mehr das Wort „error"), und ein
  Fehler vor dem ersten Token bekommt überhaupt eine Blase.
- Verifiziert mit einem echten Turn: Antwort „OK" in der Zeile. Testnachrichten
  danach aus der Datenbank entfernt.

### Verlorene Stile (separat)
- **Markdown-Blöcke** (`p`, `pre`, `code`, Listen, Überschriften) waren aus
  `MessageList.vue` verschwunden → Codeblöcke ohne `overflow-x` liefen über die
  Blase („231 行 …"). Wiederhergestellt; Tabellen scrollen in sich selbst.
- **Composer**: `.composer-wrap` war auskommentiert — das war die „Transparenz".
  `#202020` wurde nicht ignoriert, es **ist** `--bg-panel`. Jetzt: Hintergrund-
  Layer zurück, Eingabefeld nutzt `--bg-composer`.
- **Autoscroll** hing: `requestAnimationFrame` schweigt in dieser WebView (dieselbe
  Ursache wie beim Splash). Jetzt `nextTick` + `ResizeObserver` auf den Inhalt.

### Stand
Fertig: sechs Punkte aus §25–§29, Marke (Wal/Icon/Splash-Zoom), About-Seite,
MIT-Lizenz, Version 0.4.7, README + Screenshot, CI (verify + drei Plattformen +
Archive am Release), alle Regressionen oben. Offen: ein **Windows-/macOS-Smoke-Test**
(nur über CI-Artefakte möglich; lokal ist nur Linux da), `v0.4.8` taggen und den
Draft `v0.4.7` löschen.

### Die nächsten zwei Vorhaben (vom Nutzer gewünscht, noch nicht gebaut)
1. **Aktionen sichtbar machen** („UI folgt dem Begleiter"): `write`/`edit` auf eine
   Datei → Datei-Tab öffnen (Vorschau oder Monaco je nach Typ); `web_fetch` → den
   Browser auf die URL; `bash` → (Terminal-Panel fehlt noch). Mit Schalter
   „folgen" und dezentem Hinweis auf den aktiven Tab, damit die Oberfläche nicht
   ungefragt springt.
2. **Punkte-/Vertrauenssystem**: Der Begleiter darf seinen Namen früh *wählen*,
   aber der Nutzer **erfährt ihn erst ab einer Punktzahl** — Punkte für Nutzung
   über Zeit (Streaks, erledigte Aufgaben, Korrekturen, Gedächtnis-Einträge), nicht
   für einen einzelnen Klick. Eigene Tabelle, Fortschritt auf der Begleiterseite,
   kleine „Enthüllung" beim Erreichen.

### Antwort auf die Composer-Frage
Die Farbe hat keinen Alphawert. Das Wasserzeichen ist `.agent::before` — eine
eigene Ebene **hinter** den Kindern des Panels. Der Composer selbst ist opak; der
Bereich *um* die Box (der Wrap) hatte keinen Hintergrund mehr, und dort scheint
die Ebene durch. Es war nie die Farbe, es war die fehlende Fläche.

## 36. Notiert vor der Kompression (2026-10-17, fünfzehnte Runde)

Commit `f8e6a94` (Wasserzeichen) und dieser Doku-Commit. Alles, was nach der
nächsten Kompression sonst fehlen würde.

### Das Wasserzeichen lag *über* dem Composer, nicht dahinter
Der Nutzer sah die Marke durch das angeblich opake Eingabefeld. Es war kein
Durchscheinen: `.agent::before` ist **positioniert mit `z-index: auto`**, und ein
positioniertes Element malt in einer späteren Ebene als normaler In-Flow-Inhalt —
also über dem Composer. Bei 6 % Deckkraft liest sich das als Transparenz. Jetzt:
`.agent { isolation: isolate }` plus `z-index: -1` am Pseudo-Element. Die Marke
liegt hinter dem Inhalt, bleibt durch die halbtransparenten Blasen sichtbar und
verschwindet unter opaken Flächen.

### Budget: keine „Bremse"
Vom Nutzer klargestellt: keine harte Begrenzung, er „tankt lieber nach". Stattdessen
nur ein **dezenter Hinweis im Seitenmenü**, wenn das Budget überschritten ist —
der Nutzer setzt das Budget selbst. Die Anzeige wächst mit der Überschreitung
(Schriftgröße) und färbt sich weiß → gelb → orange → rot. Kein Abbruch.

### Bereits in §35 notiert (für die Umsetzung nach der Kompression)
1. **UI folgt dem Begleiter**: `write`/`edit` → Datei-Tab (Vorschau oder Monaco),
   `web_fetch` → Browser auf die URL, `bash` → Terminal (fehlt noch), `read/glob/grep`
   nur als Chip. Mit Schalter „folgen" und dezentem Hinweis statt Fokusklau.
2. **Punkte-/Vertrauenssystem**: Name früh gewählt, aber erst ab einer Punktzahl
   enthüllt; Punkte für Verhalten über Zeit (Streaks, erledigte Aufgaben,
   Korrekturen, Gedächtnispflege), nicht für einzelne Klicks; Fortschritt auf der
   Begleiterseite, kleine Enthüllung beim Erreichen. Ziggy ist bereits enthüllt —
   gilt für neue Installationen.
3. **Companion-Ideen** für später: Wiedereinstieg („Wo waren wir?"), Wochenrückblick
   aus Todos/Sitzungen, **projektgebundenes** Gedächtnis, Skill-Vorschläge
   („das machst Du zum dritten Mal von Hand"), Budgetanzeige wie oben.

### Release-Plan
Smoke-Test auf dem zweiten Linux-Rechner → README-Abschnitt „Known limitations"
(kein Editor-/Browser-Zugriff des Begleiters) → `v0.4.8` taggen → Draft `v0.4.7`
löschen. Windows/macOS nur über die CI-Artefakte (kein Testgerät).

## 37. Die Oberfläche folgt dem Begleiter (2026-10-17, sechzehnte Runde)

Umgesetzt ist das erste der beiden in §35/§36 notierten Vorhaben. Kein Commit — die
Änderungen liegen im Baum und sind mit `pnpm build`, `cargo check` und den 24
Rust-Tests geprüft.

### Was passiert
`tool_start` auf `write` oder `edit` öffnet die Datei im Datei-Tab (Vorschau oder
Monaco, wie der Typ es hergibt) und lässt den Tab kurz aufleuchten (`pulseTab`,
1,8 s). `tool_result` desselben Werkzeugs erhöht `fileRevision`; der Tab liest die
Datei daraufhin neu — ohne das hätte man den Stand *vor* dem Schreiben gesehen,
weil der Pfad sich ja nicht ändert. `web_fetch` navigiert das zuletzt benutzte
Browser-Fenster auf die URL. `read`, `glob` und `grep` bleiben Chips; `bash`
ebenfalls, das Terminal-Panel fehlt noch.

### Kein Fokusklau
Es wird nirgends `focus()` gerufen; der Cursor bleibt im Composer. Für den Browser
gibt es jetzt den optionalen Parameter `focus` an `open_browser_window`
(Standard `true` für alle bisherigen Aufrufer). Beim Folgen wird `false`
übergeben: das Fenster öffnet bzw. navigiert, drängt sich aber nicht nach vorn.

### Schalter
`settings.followCompanion` (Standard `an`), als Augensymbol rechts in der
Tab-Leiste und als Schalter unter Einstellungen → Allgemein. Aus heißt: nur die
Chips im Gespräch, kein Tab, kein Browser. Pfade werden relativ zum
Arbeitsverzeichnis aufgelöst (`resolveWorkspacePath`), absolute Pfade bleiben.

### Dateien
- `src/composables/useFollow.ts` (neu): Argumente lesen, Pfad auflösen, Tab/Browser
  folgen lassen, Pfad je Werkzeug-ID bis zum Ergebnis merken.
- `src/stores/workArea.ts`: `fileRevision`, `refreshFile`, `attentionTabId`,
  `pulseTab`.
- `src/composables/useEngine.ts`: Aufrufe an den beiden Ereignisstellen.
- `src/components/workarea/ViewFileTab.vue`: Reload-Watcher für Text, Bilder und
  Office-Vorschauen; Markdown-Ansicht springt bei neuem Pfad zurück auf Vorschau.
- `src/components/workarea/WorkArea.vue`: Augenschalter, Puls-Animation.
- `src-tauri/src/browser.rs`: `focus`-Parameter.
- i18n in allen sechs Sprachen, `Icon.vue` (Auge/Auge durchgestrichen),
  `SettingsPage.vue`, `stores/settings.ts`.

### Als Nächstes
Punkte-/Vertrauenssystem (Name gewählt, aber erst ab einer Punktzahl enthüllt),
danach Budgetanzeige im Seitenmenü.

## 38. Flüssiger streamen, Denken zuklappen, Namen begründen (2026-10-17, siebzehnte Runde)

Reaktion auf den Zwischenbericht: Das Tippen im Gespräch hakt, leere Blöcke wandern
über den Bildschirm, und die Denk-Anzeige hängt. Kein Commit; geprüft mit
`pnpm build` und den 24 Rust-Tests.

### Warum es hakte
`MessageList` wurde bei *jedem* gestreamten Bruchstück komplett neu gerendert — und
rief für **jede** fertige Nachricht erneut `renderMarkdown` samt DOMPurify auf. Bei
einer langen Unterhaltung war das quadratisch viel Arbeit pro Token. Dazu war der
Denk-Block während des Streamens aufgeklappt: jeder neue Satz erzwang ein neues
Layout eines ohnehin schon langen, mitgescrollten Textes. Die „leeren Prompts" waren
genau dieser noch leere, offene Denk-Kasten am Anfang eines Zuges.

### Was jetzt anders ist
- **`MarkdownText.vue`**: eigener Baustein mit gecachtem `computed`. Der Parent darf
  weiter rendern; nur der Block, dessen Text sich wirklich geändert hat, wird neu
  geparst und saniert. Die Text-Stile sind mit umgezogen (auch die Nutzerblase).
- **Sammelpuffer in `useEngine.ts`**: Text- und Denk-Brüche werden 40 ms lang
  gesammelt und in ihrer Reihenfolge an den Store gegeben. Jedes Ereignis, das die
  Zeitleiste ändert (Werkzeugstart/-ende, Fehler, `action_required`, `turn_end`),
  leert den Puffer vorher — ein Werkzeug-Chip kann also nicht über Text rutschen,
  der vorher geschrieben wurde. Beim Sitzungswechsel und vor einem neuen Zug wird
  ebenfalls geleert. 40 ms sind unter der Wahrnehmungsschwelle, sparen aber den
  größten Teil der Renderarbeit.
- **Denken bleibt zu**: `<details>` ohne `open`-Bindung; solange der Zug läuft, wird
  ein Aufklappen per `toggle` sofort zurückgenommen. In der Kopfzeile steht live die
  Anzahl der gedachten Zeichen (aus dem gepufferten Text, nicht geschätzt). Erst nach
  `turn_end` lässt sich der Block normal öffnen. Der Zähler bleibt sichtbar.
- Neue i18n-Zeichenkette `message.characters` in allen sechs Sprachen.

### Namen: wann und warum
`choose_name` verlangt jetzt zusätzlich `reason` (ein Satz). Nach erfolgreicher Wahl
schreibt der Rust-Teil automatisch einen Gedächtniseintrag (`kind = "event"`,
`scope = "agent"`, Quelle `choose_name`) mit dem lokalen Datum im Text — die Zeit
steht zusätzlich im `created_at` der Zeile, also im Gedächtnis-Panel. Damit kann ein
späterer Thread nicht mehr nur den Namen vorfinden, sondern auch „wann" und „warum".
(`Db::today` liefert dafür `date('now','localtime')`.) Für die bereits gewählten Namen (Ziggy,
davor Nathaniel und Eira) ist der Grund nicht rekonstruierbar — die vorhandene
Notiz sagt das ehrlich; erfunden wird nichts. `reason` fehlt → das Werkzeug antwortet
mit einer Erklärung statt die Wahl ohne Begründung zu speichern.

### Offen / als Nächstes
Punkte-/Vertrauenssystem und Budgetanzeige; danach der Smoke-Test für `v0.4.8`.

## 39. Schmales Fenster: erst die Leiste, dann der Arbeitsbereich (2026-10-17, achtzehnte Runde)

Der Build bei Mindestbreite zeigte abgeschnittene Composer-Bedienelemente („Volle
Berechtigung" brach senkrecht um, das Modellfeld lief aus der Blase) und eine
gequetschte Mitte. Reine Mindestgrößen helfen dort nicht; die Breite muss sich
anders verteilen. Kein Commit; `pnpm build` grün.

### Reihenfolge
Wie vom Nutzer vorgeschlagen, aber mit eigener, nicht gespeicherter Zustandsebene:
- `railAuto` und `workAuto` (in `stores/ui.ts`) bedeuten „gerade kein Platz", nicht
  „vom Nutzer weggeschaltet". Die manuellen Flags (`railCollapsed`, `visible`)
  bleiben unangetastet und werden weiter gespeichert.
- `useResponsiveLayout.ts` misst beim Start und bei jedem Fenster-`resize` die
  tatsächliche Mitte: erst verschwindet die linke Leiste, wenn die Mitte unter
  720 px fällt; reicht es dann immer noch nicht (unter 640 px), verschwindet der
  Arbeitsbereich. Zum Zurückholen sind die Schwellen höher (780/700 px), damit ein
  Fenster auf der Kante nicht flackert.
- Ein Klick auf die Leisten- oder Panel-Schaltfläche setzt das Auto-Flag zurück:
  wer das Panel bei schmalem Fenster ausdrücklich öffnet, bekommt es. Beim nächsten
  Resize gilt wieder die Regel.
- `openTab`/`openFile`/`showWorkArea` räumen `workAuto` ebenfalls ab — ein per
  Begleiter geöffneter File-Tab soll sichtbar sein.

### Composer und Spalte
- Gemeinsame Lese-Spalte als CSS-Variable am Agenten:
  `--chat-width: min(100%, max(70%, 520px), 860px)`. Auf weiten Fenstern bleibt es
  bei 70 %, auf schmalen wird die Spalte nicht mehr unter 520 px gedrückt; bei sehr
  breiten Fenstern begrenzt sie sich auf 860 px. Liste und Composer benutzen
  dieselbe Variable (Composer weiterhin 6 px schmaler wegen der Scrollbar-Reserve).
- `.composer` ist jetzt ein Container (`container-type: inline-size`). Unter 440 px
  eigener Breite verschwindet nur das Label von „Volle Berechtigung" (Icon und
  Klickziel bleiben), das Modellfeld schrumpft auf 118 px. Das Label kann nicht
  mehr senkrecht umbrechen (`nowrap` + Ellipse), linke und rechte Gruppe dürfen
  schrumpfen (`min-width: 0`).
- Titelzeile: Editor-Beschriftung verschwindet unter 980 px (Logo + Chevron bleiben),
  der Sitzungstitel ist auf `min(420px, 40vw)` begrenzt.
- `tauri.conf.json`: `minWidth` 960 → 780. Bei 780 px sind beide Panels automatisch
  weg, die Mitte hat ~760 px; ab ~795 px bleibt der Arbeitsbereich, ab ~1100 px
  kommt die Leiste zurück.

### Vom Nutzer am Splash
Der Wal wächst jetzt gleichmäßig (Schwanz und Maul erreichen den Rand zusammen),
ist auf 1,85 statt 2,05 skaliert, sitzt mit `top: 40px; right: 10px` und die
Splash-Leiste nutzt `--accent-whale` (#0aaac2) statt `--accent`. Unverändert
übernommen.

### Offen
Sichtprüfung bei 780/960/1100 px durch den Nutzer; Rust-Neustart nötig für
`minWidth` (Fensterkonfiguration) sowie die früheren Rust-Änderungen.

## 40. Dateibaum tritt zur Seite, Adresszeile bleibt (2026-10-17, neunzehnte Runde)

Drei Wünsche aus der Sichtprüfung des Datei-Tabs. Kein Commit; `pnpm build` grün.

### Dateibaum nach der Auswahl ausblenden
`ViewFileTab` merkt sich jetzt `fileTreeVisible` im WorkArea-Store (und damit in
`ui.workarea`, also über Neustarts hinweg). Nach einem Klick auf eine Datei im Baum
startet ein 2-s-Timer; läuft er ab, setzt er die Sichtbarkeit auf `false`. Der
Übergang selbst ist CSS: `.viewfile` animiert `grid-template-columns` von
`minmax(0,1fr) 220px` auf `minmax(0,1fr) 0px` (260 ms), die Leiste blendet dabei
aus. Wird in der Zwischenzeit eine andere Datei gewählt, beginnt der Timer neu; wird
ein Ordner auf-/zugeklappt, wird er abgebrochen — wer den Baum benutzt, soll ihn
nicht unter der Hand verlieren. Das Ausblenden ist reine Anzeige: der Baum bleibt
montiert (`inert`, `aria-hidden`), Monaco bekommt durch `automaticLayout` die neue
Breite.

### Begleiter öffnet keine Bäume mehr
Der Auto-Hide hängt nur an `select()` im Dateibaum. `openFile` (auch aus
`useFollow`) und `fileRevision` fassen die Sichtbarkeit nicht an. War der Baum zu,
bleibt er zu, wenn der Begleiter eine Datei schreibt; war er offen, bleibt er offen.

### Feste Adresszeile auch für Code
Die `.viewer-bar` wird jetzt für jede Datei gerendert, nicht nur für Markdown und
Binärformate. Bei Markdown stehen links „Vorschau/Quelle", rechts der Pfad und der
neue Baum-Schalter; bei Code (Monaco) bleibt die Zeile stehen, während der Editor
scrollt. Der frühere schwebende „Baum einblenden"-Knopf über dem Editor ist weg —
der Schalter sitzt am rechten Ende der Zeile und zeigt den Zustand (Akzentfarbe,
wenn der Baum offen ist). Damit verschwindet nichts mehr beim Scrollen, und der
Pfad ist immer sichtbar.

### Offen
Sichtprüfung durch den Nutzer (2-s-Verzögerung, Animation, Monaco-Breite).

## 41. Auch Lesen blendet den Editor ein (2026-10-17, zwanzigste Runde)

Der Begleiter konnte den Browser folgen lassen, aber keinen Editor öffnen. Ursache
war keine kaputte Verdrahtung: `useFollow` behandelte nur `write` und `edit`;
„zeig mir diese Datei" erledigt der Begleiter aber mit `read`, und das war bewusst
nur ein Chip. Damit blieb die Asymmetrie zum Browser, der über `web_fetch` folgt.

Jetzt öffnen auch `read`-Aufrufe die Datei im Datei-Tab (Vorschau oder Monaco je
Typ) und lassen den Tab kurz aufleuchten. `write`/`edit` merken sich den Pfad
weiterhin bis zum Ergebnis, damit die Vorschau den neuen Inhalt lädt; bei `read`
wird nichts nachgeladen, weil sich nichts geändert hat. `glob`, `grep` und `bash`
bleiben Chips — sie beschreiben eine Suche, kein Dokument.

Folge: Wer den Begleiter beim Durchsuchen vieler Dateien beobachtet, sieht den Tab
öfter wechseln. Dafür ist das Augensymbol in der Tab-Leiste da; aus heißt aus.
`pnpm build` grün, kein Commit.

## 42. Erinnerungen als Karten, Titel und ein weicherer Hinweis (2026-10-17, einundzwanzigste Runde)

Kein Commit; `pnpm build` grün, 24 Rust-Tests grün. Der Begleiter öffnet jetzt auch
gelesene Dateien — vom Nutzer bestätigt.

### Titel für Erinnerungen
Die `memories`-Tabelle hat eine Spalte `title` bekommen (Migration, alte Zeilen
bleiben `NULL`). Höchstens 120 Zeichen; `clean_title` faltet Weißraum und kürzt mit
Ellipse. `insert_memory_with_title` legt den Titel mit an und leitet ihn, wenn keiner
kommt, aus der ersten Zeile des Textes ab (`derive_title`). Das `memory`-Werkzeug hat
ein optionales `title`-Feld für `add` und `update`; die Befehle `memory_add` und
`memory_update` reichen es durch. Der Name-Eintrag aus §38 bekommt seinen Titel
automatisch aus der Notiz.

### Karten statt Wand
`MemoryPanel.vue`: Eingabe mit Titelzeile (Zähler 0/120) und Text. Die Liste ist ein
Raster (`auto-fill`, min. 260 px), jede Karte zeigt Titel (zwei Zeilen), Chips, eine
dreizeilige Vorschau und die Aktionen. Ein Klick öffnet ein modal über der Seite
(`Teleport to body`) mit vollem Text, Datum, Bearbeiten und Stilllegen; nichts
darunter verschiebt sich. Bearbeiten wechselt im Modal auf Titel + Text; Esc und
Klick auf den Hintergrund schließen. Alte Einträge ohne Titel zeigen ihre erste
Zeile als Überschrift. Eine Suche ist bewusst noch nicht drin — später.

### Hinweis und Sprache
„Der Agent ist KI und kann Fehler machen." ist jetzt in allen sechs Sprachen
„Auch KI-Begleiter können mal einen Fehler machen!" (bzw. „AI companions can make
mistakes too!", „Les compagnons IA peuvent eux aussi faire des erreurs !",
„¡Los compañeros de IA también pueden cometer errores!", „ИИ-собеседники тоже иногда
ошибаются!", „AI 伴侣也会犯错！").

Aus der Begriffssammlung übernommen, jeweils in der Begleiter-Seite (Hinweis und
Tagline): Deutsch „KI-Begleiter", Französisch „compagnon IA", Spanisch „compañero de
IA", Russisch „ИИ-собеседник", Chinesisch „AI 伴侣". Englisch entsprechend „AI
companion". Die kurzen Einstellungs-Labels bleiben unverändert.

### Offen
Sichtprüfung des Rasters/Modals durch den Nutzer; die Suche kommt später.

## 43. v0.4.8 (2026-10-17, zweiundzwanzigste Runde)

Der Nutzer hat den Stand geprüft und nichts mehr gefunden; daraus wird die
Veröffentlichung. Enthalten sind die Runden 37–42:

- Oberfläche folgt dem Begleiter: `write`/`edit` und jetzt auch `read` öffnen Dateien
  (Markdown-Vorschau oder Monaco), `web_fetch` den Browser — mit Augenschalter, ohne
  Fokusklau; der Dateibaum tritt nach der Auswahl zur Seite und bleibt beim
  Begleiter-Schreibzugriff zu.
- Flüssigeres Streamen: gepufferte Brüche (40 ms), `MarkdownText` mit gecachtem
  Rendering, Denken bleibt während des Zuges zu und zeigt die Zeichenzahl.
- Schmales Fenster: linke Leiste zuerst, dann der Arbeitsbereich (Auto-Flags,
  Hysterese), gemeinsame Lese-Spalte (min. 520 px), Composer-Container-Query,
  `minWidth` 780.
- Erinnerungen: Titel (max. 120 Zeichen), Kartenraster, Modal, Bearbeiten von Titel
  und Text; Migration `memories.title`; `title` im `memory`-Werkzeug.
- Namen: `choose_name` verlangt eine Begründung und legt sie datiert im Gedächtnis ab.
- Weichere Formulierung in allen sechs Sprachen („Auch KI-Begleiter können mal einen
  Fehler machen!"), Begleiter-Begriffe aus der Sammlung übernommen.
- README: Follow-Modus und Abschnitt „Known limitations"; Version in
  `package.json`, `Cargo.toml`, `tauri.conf.json` und `Cargo.lock` = 0.4.8.

Test: `pnpm build` grün, 24 Rust-Tests grün. Tag `v0.4.8` löst den Release-Workflow
aus (verify → Linux/macOS/Windows → Draft-Release mit Artefakten). Der alte Draft
`v0.4.7` wird danach gelöscht.

`docs/markdown-test.md` bleibt als lokale Testdatei unversioniert.

## 44. Der Schlüssel, der nur im Mock lag (2026-10-20, dreiundzwanzigste Runde)

Der Windows-Test hat den ersten echten plattformspezifischen Fehler gefunden: Die
App meldete „Gespeichert im OS-Schlüsselbund", verlor den API-Key aber sofort
wieder. Ursache war `keyring` 3 ohne Plattform-Features — die Crate hat keine
Defaults, und mit nur `sync-secret-service` (Linux) greift auf Windows/macOS der
plattformunabhängige Mock. Der Mock hält den Wert nur in dem `Entry`, der ihn
geschrieben hat; der nächste `Entry` in `provider_request` ist leer →
„API key is missing". Unter Linux (Secret Service) fiel das nie auf.

### Behoben
- `Cargo.toml`: `apple-native` und `windows-native` ergänzt (Linux behält
  `sync-secret-service` + `crypto-rust`). `Cargo.lock` zieht die
  plattformspezifischen Abhängigkeiten nach.
- `SecretStore::set` liest den Wert nach dem Schreiben über einen **frischen**
  `Entry` zurück. Nur wenn er exakt passt, gilt der Keyring als erfolgreich und
  die Datei-Kopie wird entfernt. Bei einem falschen Wert wird der Eintrag
  gelöscht; bei einem Lesefehler bleibt er stehen, aber der Datei-Fallback wird
  geschrieben und als `file` gemeldet. Damit ist der Mock nie wieder still.
- `keyring_roundtrip_or_skip` schreibt mit einem Entry und liest mit einem
  zweiten. Der alte Test las denselben Entry und war deshalb auch im Mock grün.
- CI: Windows/macOS führen jetzt `cargo test --lib` aus, bevor sie bauen — dort
  lebt der native Store, den der Linux-`verify`-Job nicht sehen kann.
- `skills.rs`: Windows-sichere Pfadprüfung im Test (`Path::ends_with`).
- Provider: ein gemeinsamer `reqwest::Client` für `/models` und `/user/balance`,
  ein Retry nach 500 ms bei Transportfehlern, Fehlerkette statt nur „error
  sending request"; die UI lädt Modelle und Guthaben nacheinander statt parallel
  (beobachtete Connection-Resets bei zwei gleichzeitigen Verbindungen).

### Stand
Version 0.4.9. `pnpm build` grün, `cargo test` 24 passed / 12 ignored (der
Keyring-Test lief auf Linux durch). Der Release-Workflow ist ebenfalls grün:
verify, Linux, macOS und Windows; der neue Testlauf auf Windows und macOS hat
den Keyring-Test auf beiden Plattformen wirklich ausgeführt (24 passed, nicht
übersprungen) und der Draft enthält die sieben Assets. Der Windows-11-Test mit
dem Installer ist auf dem Arbeitsrechner bereits mit einem lokalen 0.4.9-Build
durchgelaufen. v0.4.9 ist daher als **Latest** veröffentlicht (sieben Assets,
Release-Notes); der defekte v0.4.8-Draft wurde gelöscht, der Tag v0.4.8 bleibt
als Historie.

Der Editor-Picker und das Account-Pill („modest") bleiben für den nächsten
Termin liegen.

## 45. Splash-Bewegung und Marke im Rail-Fuß (2026-10-21, vierundzwanzigste Runde)

Zwei Punkte, die im v0.4.9-Release noch fehlten. Kein Commit; geprüft mit
`pnpm build`, `cargo test --lib` (24 passed, 12 ignored) und einem Windows-Build
mit `tauri/custom-protocol`.

### Der Walfisch stand unter Windows still
Ursache war nicht WebView2: Der Testrechner hat die Windows-Animationseffekte aus,
also meldet Chromium `prefers-reduced-motion: reduce`. Die Splash-Regel schaltete
daraufhin genau die Wal-Animation ab — der Fortschrittsbalken lief weiter. Der
Splash ist die einzige bewusste Bewegung der App; die Regel ist entfernt. Wal und
Balken starten außerdem erst, wenn das Fenster sichtbar ist (`splashShown`), statt
schon beim Laden der versteckten Webview. Die sieben Sekunden Animation und die
sieben Sekunden Splash laufen damit exakt zusammen.

### Marke im Rail-Fuß
Der Fuß war fest auf „modest" und einen Sparkle-Avatar verdrahtet. Walfisch und
„deepseek"-Schriftzug sind jetzt dort — vom Kopf des Panels dorthin gezogen; der
Fuß bleibt der Anker für das Konto-Menü (Chevron). Der Benutzername steht
weiterhin im Menükopf, nicht mehr statisch im Panel.

### Offen
v0.4.10 taggen.

## 46. Der Editor-Picker öffnet den Workspace (2026-10-21, fünfundzwanzigste Runde)

`open_workspace_in_editor` lag seit §5 ungenutzt im Rust-Teil; `openInEditor()` war
exportiert, aber niemand rief es auf. Jetzt ist die Pille ein Split-Button: links
öffnet den Workspace im gewählten Editor, rechts öffnet die Auswahl. Ohne
gefundenen Editor öffnet der linke Teil die Auswahl, damit Erkennen und Download
erreichbar bleiben; ein Fehler beim Start erscheint fünf Sekunden unter der Pille.
`ensureWorkspace()` sorgt dafür, dass ein leerer Workspace nicht als „Editor ohne
Ordner" endet.

Der Windows-Test hat zwei echte Fehler im alten Befehl gefunden:
- Der PATH-Eintrag `C:\Program Files\Microsoft VS Code\bin\code` ist ein Pfad mit
  Leerzeichen, wurde aber durch denselben Command-Splitter wie die Desktop-Einträge
  geschickt → als Programm blieb `C:\Program` (os error 2). PATH-Kandidaten werden
  jetzt gequotet; auf Windows bleibt der Pfad außerdem un-canonicalisiert (kein
  `\\?\`-Präfix).
- Der gefundene `code` ist der Unix-Shim; daneben liegt `code.cmd`, den Windows
  nicht direkt startet. Der Launcher nimmt auf Windows den `.cmd`/`.bat`-Shim über
  `cmd /C` (mit `CREATE_NO_WINDOW`) und nennt sonst den Programmnamen im Fehler.

Geprüft: `pnpm build`, `cargo test --lib` (24 passed, 12 ignored), Windows-Release
mit `tauri/custom-protocol`; ein kalter VS-Code-Start über den Split-Button öffnete
`X:\DEV\dside-main` (Fenstertitel „Willkommen - dside-main").

## 47. Dateiverlauf mit Diff und Wiederherstellen (2026-10-21, sechsundzwanzigste Runde)

Der wichtigste Vertrauensbaustein: Der Partner kann jetzt zeigen, was er geschrieben hat,
und der Nutzer kann es zurücknehmen.

### Snapshots
- Neue Tabelle `file_versions` (Pfad, Inhalt, existed, Session, Herkunft, Call-ID, Zeit).
  `content NULL` heißt: Die Datei existierte vorher nicht; Wiederherstellen löscht sie.
- `engine/mod.rs` liest den alten Stand vor jedem `write`/`edit` und schreibt den
  Snapshot erst, wenn das Werkzeug erfolgreich war. Das gilt auch für Sub-Agenten
  (`task`), die dafür AppHandle und Session-ID bekommen.
- `tools::stable_path` normalisiert Pfade (Windows-Separatoren, kein `\\?\`), damit
  Dateibaum, Modell und Verlauf denselben Schlüssel benutzen.

### UI
- Datei-Tab: „Verlauf" öffnet links die Liste (neueste zuerst, mit Zeitpunkt, Herkunft
  Schreiben/Bearbeiten/Wiederherstellen und „Neu angelegt"), rechts einen read-only
  Monaco-Diff gegen den aktuellen Stand.
- „Wiederherstellen" sichert vorher den aktuellen Stand als weiteren Snapshot und
  schreibt dann die gewählte Fassung; ein Snapshot von vor der Existenz der Datei
  entfernt sie wieder.
- Ein `write`/`edit`-Chip hat im aufgeklappten Detail einen „Verlauf"-Knopf; der
  Datei-Tab öffnet sich mit dem Verlauf.
- Fehler beim Laden/Wiederherstellen erscheinen im Panel; die Bestätigung fragt vor
  dem Überschreiben.

### Tests
- `db::tests::file_versions_round_trip` (Liste, neueste zuerst, NULL/created).
- `tests::reverting_a_snapshot_restores_and_backs_up_the_current_state` (Restore,
  Sicherung, Löschen eines vorher nicht existierenden Snapshots).
- Live auf Windows: Der Partner schrieb `history-test.txt` (Snapshot `write`,
  existed=0), änderte `eins`→`zwei` (Snapshot `edit`, alter Inhalt `eins`); der Verlauf
  zeigte beide, der Diff `eins`→`zwei`, der Restore des „Neu angelegt"-Snapshots
  entfernte die Datei. `cargo test --lib` 26 passed / 12 ignored; `pnpm build` grün.

## 48. AGENTS.md: die Regeln des Repos (2026-10-21, siebenundzwanzigste Runde)

Der Partner kannte bisher Persona, Nutzeranweisungen und Gedächtnis, aber nicht die
Konventionen des Projekts. Jetzt liest der Engine-Loop beim Start eines Turns `AGENTS.md`
(Fallback `.dside/rules.md`) aus dem Workspace und hängt sie an den System-Prompt;
Sub-Agenten bekommen dieselben Regeln. Der Text ist auf 20.000 Zeichen begrenzt, damit
eine ausufernde Datei nicht das Kontextfenster frisst.

Die Datei im Wurzelverzeichnis ist gleichzeitig die Anleitung für dieses Repository:
pnpm 10, Build-/Testbefehle, PROGRESS als Chronik, Schlüsselbund, die sechs Sprachen und
die Lizenzhinweise.

Test: `project_instructions_are_read_from_the_workspace` (Temp-AGENTS.md gelesen, Quelle
im Prompt). `cargo test --lib` 27 passed / 12 ignored.

## 49. Doctor und Erststart (2026-10-21, achtundzwanzigste Runde)

### Diagnose
- Einstellungen → Diagnose: Version, System, Keyring-Backend, Workspace, Anbieter/Modell,
  Datenbankpfad/-größe und erkannte Editoren. „Verbindung testen" lädt Modelle und
  Guthaben seriell (dieselbe Ursache wie in §44), „Diagnose kopieren" legt den Text in
  die Zwischenablage.
- Neuer Rust-Befehl `diagnostics` liest Keyring-Status, Settings und DB-Größe; der
  Provider-Test bleibt der einzige Netzschritt und wird von der UI ausgelöst.

### Erststart
- Ist die `settings`-Tabelle leer (frische Installation), erscheint ein Dialog: Modus
  (Programmierung/Alltag), Workspace (über `read_workspace_tree` validiert) und API-Key
  (OS-Schlüsselbund). „Später" setzt `onboarded` und zeigt den Dialog nicht wieder.
- Bestehende Installationen — auch solche ohne den neuen `onboarded`-Schlüssel — sehen
  den Dialog nie.
- Dabei einen älteren Sync-Fehler behoben: Der Modell-Select im Composer hing an einem
  lokalen Ref und folgte einem Moduswechsel oder dem Erststart nicht; er synchronisiert
  jetzt mit `settings.model`.

### Tests
- Live auf Windows mit geleerter Settings-Tabelle: Dialog erschien, „Für Programmierung"
  → „Los"; die Diagnose zeigte Keyring `keyring`, DB 80 KB, VS Code; der Verbindungstest
  meldete „2 Modelle · 27.58 USD".
- `pnpm build`/vue-tsc grün, `cargo test --lib` 27 passed / 12 ignored, Windows-Release
  mit `tauri/custom-protocol`. Die ursprüngliche Datenbank (Settings, Session, Workspace)
  wurde nach dem Test wiederhergestellt.
- Das Workspace-Feld wurde nach dem Live-Test ergänzt; die Validierung nutzt denselben
  `read_workspace_tree`-Befehl wie der Dateibaum.

## 50. Übergabeunterlage (2026-10-21, neunundzwanzigste Runde)

`HANDOVER.md` liegt jetzt im Wurzelverzeichnis: Architektur, Build/Dev/Release (inklusive
pnpm 10 und dem Windows-`custom-protocol`-Feature), Erststart, Speicherorte (SQLite,
Keyring, Attachments, Skills), Vertrauensmodell, bekannte Grenzen (kein Terminal/Git,
keine Signierung oder Updater, macOS-aarch64, keine Memory-Suche, kein Screenshot-Tool)
und die ersten Prüfschritte für neue Betreuer. Der Fresh-Install-Smoke-Test steht in §49.

## 51. Retry, Basis-URL und der idempotente Boot (2026-10-21, dreißigste Runde)

- Fehlgeschlagene Turns haben jetzt einen **„Erneut versuchen"**-Knopf. `engine_retry`
  rollt die Nachrichten und Tool-Calls nach der letzten User-Nachricht zurück, lässt die
  User-Nachricht stehen und startet denselben Turn neu, ohne sie ein zweites Mal
  einzufügen. Der Titel bleibt; fehlt eine User-Nachricht (Fehler vor dem Persistieren),
  meldet der Befehl das ehrlich.
- Die Fehlerblase überlebt jetzt einen Besuch in den Einstellungen: `initEngine` ist
  idempotent. Vorher lud das erneut montierte AgentPanel die gespeicherte Historie und
  verlor die Live-Fehlermeldung samt Retry-Knopf.
- Die **Basis-URL** ist endlich unter Einstellungen → Konto & Modell editierbar. Der Wert
  war im Engine-Config schon vorhanden, hatte aber kein Feld.
- Tests: `rollback_removes_the_failed_tail_but_keeps_the_user_message`; insgesamt
  **28 passed / 12 ignored**. Live auf Windows: Basis-URL auf einen 404-Pfad gestellt,
  Fehlerblase erschien; URL korrigiert, Settings geschlossen, Retry geklickt → Antwort
  „RETRY", der fehlgeschlagene Assistant-Row war aus der DB entfernt und die
  User-Nachricht stand genau einmal. Die ursprüngliche Datenbank wurde danach
  zurückgespielt.

## 52. Projektgedächtnis und „Wo waren wir" (2026-10-21, einunddreißigste Runde)

- `memories.workspace` (Migration, Default `''`): Notizen ohne Workspace gelten überall,
  Notizen mit einem Pfad nur in diesem Projekt. Das `memory`-Werkzeug hängt beim `add`
  den aktuellen `config.workspace` an; der Recall (`memory_context`) nimmt globale plus
  die des aktuellen Workspace; die Prompt-Zeilen markieren Projektnotizen mit
  `(project)`. Das Erinnerungs-Panel zeigt einen „Projekt"-Chip, der Pfad steht im
  Tooltip.
- `session_resume` liefert die Kompressions-Zusammenfassung. Die Nachrichtenliste zeigt
  sie als **sticky Karte „Wo waren wir?"** über dem Verlauf; `loadHistory` holt sie, die
  manuelle Verdichtung lädt neu. Sechs Sprachen.
- Tests: `memories_are_scoped_to_the_workspace`; **29 passed / 12 ignored**.
- Live: Der Partner legte „Die Testfarbe … Blau" per memory-Werkzeug mit Workspace
  `X:\DEV\dside-main` ab (Ausgabe „project"). Eine **neue Session ohne diesen Verlauf**
  antwortete „Blau. Das steht so in meinen Notizen zu diesem Projekt." Das Panel zeigte
  den Projekt-Chip; die eingefügte Zusammenfassung erschien als sticky
  „Wo waren wir?"-Karte. Die Original-DB wurde danach zurückgespielt.

## 53. Zuhause integriert und geprüft (2026-10-21, zweiunddreißigste Runde)

Das Handover aus Runde 45–52 ist sauber auf `main` (nach v0.4.9) angewendet
worden: `dside-ui-fix.patch` lief ohne Konflikt durch, die drei neuen Komponenten
(Erststart, Diagnose, Monaco-Diff), `AGENTS.md` und `HANDOVER.md` sind im Baum.

Geprüft auf Linux: `pnpm build` grün, `cargo test --lib` **29 passed / 12
ignored** (inklusive Dateiverlauf- und Workspace-Gedächtnis-Test). Version steht
in allen Manifesten auf 0.4.10. Der v0.4.10-Tag ist gepusht; der Release-Workflow
ist grün (verify, Linux, macOS, Windows, inklusive der neuen Bibliothekstests auf
Windows/macOS) und hat den Draft mit sieben Assets angelegt.

**v0.4.9 bleibt der veröffentlichte Stand**; v0.4.10 liegt als Draft bereit und
wird erst nach der Sichtprüfung veröffentlicht.

## 54. Tastenkürzel: sichtbar, änderbar, konfliktgeprüft (2026-10-21, dreiunddreißigste Runde)

Der Eintrag „Tastenkürzel" in den Einstellungen war bisher nur ein Menüpunkt ohne
Seite. Jetzt gibt es eine echte Liste — nicht die Vorlage eins zu eins, sondern
die Befehle, die die App wirklich hat.

### Registry statt verstreuter Handler
`src/engine/shortcuts.ts` beschreibt jeden Befehl einmal: ID, i18n-Label, Gruppe,
Geltungsbereich, Standardbelegung. `bindingFor` macht aus einem Tastaturereignis
eine kanonische Zeichenkette (`Ctrl+Alt+B`, `Alt+ArrowUp`), Cmd zählt auf macOS
als Ctrl. `activeBindings` liest die Overrides aus `settings.shortcuts`; ohne
Override gelten die Defaults. `bindingOwner` findet Konflikte, `setBindings` /
`resetCommand` / `resetAllBindings` schreiben die Overrides.

### Was tatsächlich belegt ist
Global und editierbar: Suchen (`Ctrl+K`, `Ctrl+Shift+P`), Einstellungen (`Ctrl+,`),
neue Aufgabe (`Ctrl+N`), Seitenleiste (`Ctrl+B`), Arbeitsbereich an/aus
(`Ctrl+Alt+B`), Arbeitsbereich öffnen (`Ctrl+O`), Tab schließen (`Ctrl+W`),
vorherige/nächste Aufgabe (`Alt+ArrowUp/Down`), Modus (`Ctrl+Shift+M`), Denkstufe
(`Ctrl+T`), Design (`Ctrl+Shift+L`). Fest im Composer: `Enter` sendet,
`Shift+Enter` macht den Zeilenumbruch. Bewusst **nicht** aufgenommen: Terminal,
Zoom, Zurück/Vorwärts und Modellmenü — dafür fehlen die Funktionen; tote Kürzel
wären schlimmer als keine.

### Panel
`KeyboardPanel.vue` zeigt die Tabelle mit Suche, „Alle auf Standard
zurücksetzen", Belegungs-Chips und Geltungsbereich. Ein Klick auf einen Chip
öffnet den Capture-Dialog; die nächste Kombination wird übernommen, `Esc` bricht
ab, `Backspace` leert die Eingabe. Globale Kürzel brauchen Ctrl/Alt/Cmd, sonst
würde Tippen sie auslösen. Konflikte mit einem anderen Befehl blockieren das
Speichern und nennen den Besitzer. Einzelne Belegungen lassen sich über das ×
entfernen; der Reset stellt die Defaults wieder her. Alles in sechs Sprachen.

`useShortcuts` ist der eine globale Listener (App.vue); der alte harte
`Ctrl+K`-Handler ist weg. Während der Capture-Dialog offen ist, pausiert er über
`shortcutCapturing`.

### Geprüft
`pnpm build` (vue-tsc + Vite) grün. Eine echte Sichtprüfung des Panels und der
Tastenkombinationen steht noch aus; die Belegungen sind bewusst so gewählt, dass
sie auf deutschen und US-Layouts funktionieren.

## 55. Der Composer folgt dem hellen Design (2026-10-21, vierunddreißigste Runde)

Im hellen Theme blieb die Eingabeblase dunkel: `--bg-composer` war nur im dunklen
Root definiert, und die Light-Variablen kannten die Variable nicht. Dadurch fiel sie
auf `#202020` zurück, während `color-scheme` schon hell war — daher der schwarze
Kasten mit weißem Placeholder und dem weißen Modell-Select.

Light setzt `--bg-composer` jetzt auf `#f7f7f7` (dieselbe Fläche wie `--bg-panel`);
Border, Text und der Select bleiben unverändert und passen nun zusammen. Das dunkle
Theme behält `#202020`. `pnpm build` grün; noch nicht committet, gehört zum
v0.4.11-Stand.
