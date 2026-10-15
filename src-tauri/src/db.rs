use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS sessions (
  id          TEXT PRIMARY KEY,
  title       TEXT NOT NULL DEFAULT '',
  grp         TEXT NOT NULL DEFAULT 'project',
  workspace   TEXT NOT NULL DEFAULT '',
  created_at  INTEGER NOT NULL,
  updated_at  INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS messages (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id  TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
  role        TEXT NOT NULL,
  content     TEXT,
  tool_calls  TEXT,
  tool_call_id TEXT,
  created_at  INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id, id);

CREATE TABLE IF NOT EXISTS tool_calls (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id  TEXT NOT NULL,
  message_id  INTEGER,
  call_id     TEXT NOT NULL,
  name        TEXT NOT NULL,
  arguments   TEXT,
  output      TEXT,
  is_error    INTEGER NOT NULL DEFAULT 0,
  duration_ms INTEGER,
  created_at  INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_tool_calls_message ON tool_calls(message_id);

CREATE TABLE IF NOT EXISTS todos (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id  TEXT NOT NULL,
  content     TEXT NOT NULL,
  status      TEXT,
  priority    TEXT,
  created_at  INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_todos_session ON todos(session_id);

-- Long-term memory. Filled deliberately, not by default. See PROGRESS.md.
CREATE TABLE IF NOT EXISTS memories (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  kind         TEXT NOT NULL,
  scope        TEXT NOT NULL DEFAULT 'user',
  text         TEXT NOT NULL,
  source       TEXT,
  importance   INTEGER NOT NULL DEFAULT 1,
  created_at   INTEGER NOT NULL,
  last_used_at INTEGER
);
CREATE INDEX IF NOT EXISTS idx_memories_scope ON memories(scope, kind);

CREATE TABLE IF NOT EXISTS settings (
  key    TEXT PRIMARY KEY,
  value  TEXT NOT NULL
);

-- Interval scheduler. Deliberately not full cron: an interval is what the UI offers,
-- and pretending otherwise would be dishonest.
CREATE TABLE IF NOT EXISTS schedules (
  id               INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id       TEXT NOT NULL,
  prompt           TEXT NOT NULL,
  interval_seconds INTEGER NOT NULL,
  enabled          INTEGER NOT NULL DEFAULT 1,
  last_run_at      INTEGER,
  next_run_at      INTEGER NOT NULL,
  created_at       INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_schedules_due ON schedules(enabled, next_run_at);
PRAGMA user_version = 1;
";

pub fn unix_now() -> i64 {
    now()
}

fn row_to_schedule(row: &rusqlite::Row<'_>) -> rusqlite::Result<ScheduleRow> {
    Ok(ScheduleRow {
        id: row.get(0)?,
        session_id: row.get(1)?,
        prompt: row.get(2)?,
        interval_seconds: row.get(3)?,
        enabled: row.get::<_, i32>(4)? != 0,
        last_run_at: row.get(5)?,
        next_run_at: row.get(6)?,
    })
}

const SCHEDULE_COLUMNS: &str =
    "id, session_id, prompt, interval_seconds, enabled, last_run_at, next_run_at";

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

/// Cut a readable window around the first match, so hits make sense in a list.
fn snippet_around(text: &str, needle: &str, width: usize) -> String {
    let lowered = text.to_lowercase();
    let needle_lower = needle.to_lowercase();
    let chars: Vec<char> = text.chars().collect();

    let Some(position) = lowered.find(&needle_lower) else {
        return chars.into_iter().take(width).collect();
    };

    let prefix_chars = text[..position].chars().count();
    let start = prefix_chars.saturating_sub(width / 3);
    let end = (start + width).min(chars.len());

    let mut out = String::new();
    if start > 0 {
        out.push('…');
    }
    out.extend(chars[start..end].iter());
    if end < chars.len() {
        out.push('…');
    }
    out.replace('\n', " ")
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRow {
    pub id: String,
    pub title: String,
    pub group: String,
    pub workspace: String,
    pub updated_at: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolRow {
    pub id: String,
    pub name: String,
    pub arguments: Option<String>,
    pub output: Option<String>,
    pub is_error: bool,
    pub duration_ms: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryMessage {
    pub id: String,
    pub role: String,
    pub text: String,
    pub tools: Vec<ToolRow>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoRow {
    pub id: i64,
    pub content: String,
    pub status: Option<String>,
    pub priority: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleRow {
    pub id: i64,
    pub session_id: String,
    pub prompt: String,
    pub interval_seconds: i64,
    pub enabled: bool,
    pub last_run_at: Option<i64>,
    pub next_run_at: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub kind: String,
    pub session_id: String,
    pub session_title: String,
    pub snippet: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryReport {
    pub total: i64,
    /// Entries that repeat an older one (exact after normalising whitespace and case).
    pub duplicates: Vec<i64>,
    /// Never used and older than the cutoff — candidates for forgetting.
    pub stale: Vec<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryRow {
    pub id: i64,
    pub kind: String,
    pub scope: String,
    pub text: String,
    pub importance: i64,
    pub created_at: i64,
}

pub struct Db {
    conn: Mutex<Connection>,
}

impl Db {
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let conn = Connection::open(path).map_err(|error| error.to_string())?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|error| error.to_string())?;
        conn.execute_batch(SCHEMA).map_err(|error| error.to_string())?;

        // The database contains prompts, tool output and file content: keep it private.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
        }

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn lock(&self) -> Result<MutexGuard<'_, Connection>, String> {
        self.conn.lock().map_err(|error| error.to_string())
    }

    pub fn ensure_session(
        &self,
        id: &str,
        title: &str,
        group: &str,
        workspace: &str,
    ) -> Result<(), String> {
        let conn = self.lock()?;
        let stamp = now();
        conn.execute(
            "INSERT INTO sessions (id, title, grp, workspace, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5)
             ON CONFLICT(id) DO UPDATE SET
               updated_at = ?5,
               workspace  = ?4,
               title      = CASE WHEN ?2 = '' THEN sessions.title ELSE ?2 END",
            params![id, title, group, workspace, stamp],
        )
        .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn touch_session(&self, id: &str) -> Result<(), String> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE sessions SET updated_at = ?2 WHERE id = ?1",
            params![id, now()],
        )
        .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn session_title(&self, id: &str) -> Result<Option<String>, String> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT title FROM sessions WHERE id = ?1",
            params![id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|error| error.to_string())
    }

    pub fn update_session(
        &self,
        id: &str,
        title: Option<&str>,
        group: Option<&str>,
    ) -> Result<(), String> {
        let conn = self.lock()?;
        let stamp = now();
        if let Some(title) = title {
            conn.execute(
                "UPDATE sessions SET title = ?2, updated_at = ?3 WHERE id = ?1",
                params![id, title, stamp],
            )
            .map_err(|error| error.to_string())?;
        }
        if let Some(group) = group {
            conn.execute(
                "UPDATE sessions SET grp = ?2, updated_at = ?3 WHERE id = ?1",
                params![id, group, stamp],
            )
            .map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    pub fn list_sessions(&self) -> Result<Vec<SessionRow>, String> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(
                "SELECT id, title, grp, workspace, updated_at
                 FROM sessions ORDER BY updated_at DESC",
            )
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([], |row| {
                Ok(SessionRow {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    group: row.get(2)?,
                    workspace: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())
    }

    pub fn create_session(&self, id: &str, title: &str, workspace: &str) -> Result<(), String> {
        self.ensure_session(id, title, "project", workspace)
    }

    pub fn delete_session(&self, id: &str) -> Result<(), String> {
        let conn = self.lock()?;
        conn.execute("DELETE FROM messages WHERE session_id = ?1", params![id])
            .map_err(|error| error.to_string())?;
        conn.execute("DELETE FROM tool_calls WHERE session_id = ?1", params![id])
            .map_err(|error| error.to_string())?;
        conn.execute("DELETE FROM todos WHERE session_id = ?1", params![id])
            .map_err(|error| error.to_string())?;
        conn.execute("DELETE FROM sessions WHERE id = ?1", params![id])
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn insert_message(
        &self,
        session_id: &str,
        role: &str,
        content: Option<&str>,
        tool_calls: Option<&str>,
        tool_call_id: Option<&str>,
    ) -> Result<i64, String> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO messages (session_id, role, content, tool_calls, tool_call_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![session_id, role, content, tool_calls, tool_call_id, now()],
        )
        .map_err(|error| error.to_string())?;
        Ok(conn.last_insert_rowid())
    }

    pub fn insert_tool_call(
        &self,
        session_id: &str,
        message_id: Option<i64>,
        call_id: &str,
        name: &str,
        arguments: Option<&str>,
        output: Option<&str>,
        is_error: bool,
        duration_ms: Option<i64>,
    ) -> Result<(), String> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO tool_calls
               (session_id, message_id, call_id, name, arguments, output, is_error, duration_ms, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                session_id,
                message_id,
                call_id,
                name,
                arguments,
                output,
                is_error as i32,
                duration_ms,
                now()
            ],
        )
        .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn replace_todos(&self, session_id: &str, todos: &[serde_json::Value]) -> Result<(), String> {
        let conn = self.lock()?;
        conn.execute("DELETE FROM todos WHERE session_id = ?1", params![session_id])
            .map_err(|error| error.to_string())?;
        let stamp = now();
        for todo in todos {
            let content = todo
                .get("content")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            if content.is_empty() {
                continue;
            }
            let status = todo.get("status").and_then(serde_json::Value::as_str);
            let priority = todo.get("priority").and_then(serde_json::Value::as_str);
            conn.execute(
                "INSERT INTO todos (session_id, content, status, priority, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![session_id, content, status, priority, stamp],
            )
            .map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    pub fn list_todos(&self, session_id: &str) -> Result<Vec<TodoRow>, String> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(
                "SELECT id, content, status, priority FROM todos
                 WHERE session_id = ?1 ORDER BY id",
            )
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([session_id], |row| {
                Ok(TodoRow {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    status: row.get(2)?,
                    priority: row.get(3)?,
                })
            })
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())
    }

    fn tools_for(conn: &Connection, message_id: i64) -> Result<Vec<ToolRow>, String> {
        let mut statement = conn
            .prepare(
                "SELECT call_id, name, arguments, output, is_error, duration_ms
                 FROM tool_calls WHERE message_id = ?1 ORDER BY id",
            )
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([message_id], |row| {
                Ok(ToolRow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    arguments: row.get(2)?,
                    output: row.get(3)?,
                    is_error: row.get::<_, i32>(4)? != 0,
                    duration_ms: row.get(5)?,
                })
            })
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())
    }

    pub fn load_history(&self, session_id: &str) -> Result<Vec<HistoryMessage>, String> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(
                "SELECT id, role, content FROM messages
                 WHERE session_id = ?1 AND role IN ('user', 'assistant')
                 ORDER BY id",
            )
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([session_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            })
            .map_err(|error| error.to_string())?;

        let mut history = Vec::new();
        for row in rows {
            let (id, role, content) = row.map_err(|error| error.to_string())?;
            history.push(HistoryMessage {
                id: format!("m{id}"),
                role,
                text: content.unwrap_or_default(),
                tools: Self::tools_for(&conn, id)?,
            });
        }
        Ok(history)
    }

    pub fn load_llm_messages(
        &self,
        session_id: &str,
    ) -> Result<Vec<crate::engine::types::ChatMessage>, String> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(
                "SELECT role, content, tool_calls, tool_call_id FROM messages
                 WHERE session_id = ?1 ORDER BY id",
            )
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([session_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            })
            .map_err(|error| error.to_string())?;

        let mut messages = Vec::new();
        for row in rows {
            let (role, content, tool_calls, tool_call_id) = row.map_err(|error| error.to_string())?;
            let parsed_calls = tool_calls
                .as_deref()
                .and_then(|raw| serde_json::from_str::<Vec<crate::engine::types::OutgoingToolCall>>(raw).ok());
            messages.push(crate::engine::types::ChatMessage {
                role,
                content,
                tool_calls: parsed_calls,
                tool_call_id,
            });
        }
        Ok(messages)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), String> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = ?2",
            params![key, value],
        )
        .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn all_settings(&self) -> Result<HashMap<String, String>, String> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare("SELECT key, value FROM settings")
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|error| error.to_string())?;

        let mut map = HashMap::new();
        for row in rows {
            let (key, value) = row.map_err(|error| error.to_string())?;
            map.insert(key, value);
        }
        Ok(map)
    }

    // -----------------------------------------------------------------------
    // Interval scheduler
    // -----------------------------------------------------------------------

    pub fn list_schedules(&self) -> Result<Vec<ScheduleRow>, String> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(&format!(
                "SELECT {SCHEDULE_COLUMNS} FROM schedules ORDER BY next_run_at"
            ))
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([], row_to_schedule)
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())
    }

    pub fn create_schedule(
        &self,
        session_id: &str,
        prompt: &str,
        interval_seconds: i64,
    ) -> Result<i64, String> {
        let stamp = now();
        let interval = interval_seconds.max(60);
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO schedules
               (session_id, prompt, interval_seconds, enabled, next_run_at, created_at)
             VALUES (?1, ?2, ?3, 1, ?4, ?5)",
            params![session_id, prompt, interval, stamp + interval, stamp],
        )
        .map_err(|error| error.to_string())?;
        Ok(conn.last_insert_rowid())
    }

    pub fn delete_schedule(&self, id: i64) -> Result<(), String> {
        let conn = self.lock()?;
        conn.execute("DELETE FROM schedules WHERE id = ?1", params![id])
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn set_schedule_enabled(&self, id: i64, enabled: bool) -> Result<(), String> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE schedules SET enabled = ?2 WHERE id = ?1",
            params![id, enabled as i32],
        )
        .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn due_schedules(&self, at: i64) -> Result<Vec<ScheduleRow>, String> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(&format!(
                "SELECT {SCHEDULE_COLUMNS} FROM schedules
                 WHERE enabled = 1 AND next_run_at <= ?1 ORDER BY next_run_at"
            ))
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([at], row_to_schedule)
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())
    }

    pub fn mark_schedule_run(&self, id: i64, next_run_at: i64) -> Result<(), String> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE schedules SET last_run_at = ?2, next_run_at = ?3 WHERE id = ?1",
            params![id, now(), next_run_at],
        )
        .map_err(|error| error.to_string())?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Search
    // -----------------------------------------------------------------------

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchHit>, String> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }
        let needle = format!("%{trimmed}%");
        let conn = self.lock()?;
        let mut hits: Vec<SearchHit> = Vec::new();

        let mut sessions = conn
            .prepare(
                "SELECT id, title FROM sessions
                 WHERE title LIKE ?1 ORDER BY updated_at DESC LIMIT ?2",
            )
            .map_err(|error| error.to_string())?;
        let rows = sessions
            .query_map(params![needle, limit as i64], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|error| error.to_string())?;
        for row in rows {
            let (id, title) = row.map_err(|error| error.to_string())?;
            hits.push(SearchHit {
                kind: "session".to_string(),
                session_id: id,
                snippet: title.clone(),
                session_title: title,
            });
        }

        let mut messages = conn
            .prepare(
                "SELECT m.session_id, s.title, m.content
                 FROM messages m JOIN sessions s ON s.id = m.session_id
                 WHERE m.content LIKE ?1 AND m.role IN ('user', 'assistant')
                 ORDER BY m.id DESC LIMIT ?2",
            )
            .map_err(|error| error.to_string())?;
        let rows = messages
            .query_map(params![needle, limit as i64], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            })
            .map_err(|error| error.to_string())?;
        for row in rows {
            let (session_id, session_title, content) = row.map_err(|error| error.to_string())?;
            hits.push(SearchHit {
                kind: "message".to_string(),
                session_id,
                session_title,
                snippet: snippet_around(&content.unwrap_or_default(), trimmed, 140),
            });
        }

        hits.truncate(limit);
        Ok(hits)
    }

    // -----------------------------------------------------------------------
    // Long-term memory
    // -----------------------------------------------------------------------

    pub fn insert_memory(
        &self,
        kind: &str,
        scope: &str,
        text: &str,
        source: Option<&str>,
    ) -> Result<i64, String> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO memories (kind, scope, text, source, importance, created_at)
             VALUES (?1, ?2, ?3, ?4, 1, ?5)",
            params![kind, scope, text, source, now()],
        )
        .map_err(|error| error.to_string())?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_memories(&self, limit: usize) -> Result<Vec<MemoryRow>, String> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(
                "SELECT id, kind, scope, text, importance, created_at
                 FROM memories ORDER BY created_at DESC LIMIT ?1",
            )
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([limit as i64], |row| {
                Ok(MemoryRow {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    scope: row.get(2)?,
                    text: row.get(3)?,
                    importance: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())
    }

    /// Most valuable entries first: importance, then freshness. Used for recall.
    pub fn memory_context(&self, limit: usize) -> Result<Vec<MemoryRow>, String> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(
                "SELECT id, kind, scope, text, importance, created_at
                 FROM memories
                 ORDER BY importance DESC, COALESCE(last_used_at, 0) DESC, created_at DESC
                 LIMIT ?1",
            )
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([limit as i64], |row| {
                Ok(MemoryRow {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    scope: row.get(2)?,
                    text: row.get(3)?,
                    importance: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())
    }

    pub fn touch_memories(&self, ids: &[i64]) -> Result<(), String> {
        if ids.is_empty() {
            return Ok(());
        }
        let conn = self.lock()?;
        let stamp = now();
        for id in ids {
            conn.execute(
                "UPDATE memories SET last_used_at = ?2 WHERE id = ?1",
                params![id, stamp],
            )
            .map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    pub fn delete_memory(&self, id: i64) -> Result<(), String> {
        let conn = self.lock()?;
        conn.execute("DELETE FROM memories WHERE id = ?1", params![id])
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    /// The mechanical half of curation: what is provably redundant, and what has gone
    /// unused for a long time. Judgement about *meaning* stays with the model or the user —
    /// this only proposes, it never deletes on its own.
    pub fn memory_report(&self, stale_after_days: i64) -> Result<MemoryReport, String> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(
                "SELECT id, text, COALESCE(last_used_at, 0), created_at
                 FROM memories ORDER BY id",
            )
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            })
            .map_err(|error| error.to_string())?;

        let cutoff = now() - stale_after_days.max(1) * 86_400;
        let mut seen: HashMap<String, i64> = HashMap::new();
        let mut duplicates = Vec::new();
        let mut stale = Vec::new();
        let mut total = 0i64;

        for row in rows {
            let (id, text, last_used, created_at) = row.map_err(|error| error.to_string())?;
            total += 1;

            let key = text
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .to_lowercase();
            match seen.get(&key) {
                Some(_) => duplicates.push(id),
                None => {
                    seen.insert(key, id);
                }
            }

            if last_used == 0 && created_at < cutoff {
                stale.push(id);
            }
        }

        Ok(MemoryReport {
            total,
            duplicates,
            stale,
        })
    }
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn sessions_list(db: tauri::State<'_, Db>) -> Result<Vec<SessionRow>, String> {
    db.list_sessions()
}

#[tauri::command]
pub fn session_create(
    db: tauri::State<'_, Db>,
    id: String,
    title: String,
    workspace: String,
) -> Result<(), String> {
    db.create_session(&id, &title, &workspace)
}

#[tauri::command]
pub fn session_delete(db: tauri::State<'_, Db>, id: String) -> Result<(), String> {
    db.delete_session(&id)
}

#[tauri::command]
pub fn session_update(
    db: tauri::State<'_, Db>,
    id: String,
    title: Option<String>,
    group: Option<String>,
) -> Result<(), String> {
    db.update_session(&id, title.as_deref(), group.as_deref())
}

#[tauri::command]
pub fn session_history(
    db: tauri::State<'_, Db>,
    session_id: String,
) -> Result<Vec<HistoryMessage>, String> {
    db.load_history(&session_id)
}

#[tauri::command]
pub fn session_todos(db: tauri::State<'_, Db>, session_id: String) -> Result<Vec<TodoRow>, String> {
    db.list_todos(&session_id)
}

#[tauri::command]
pub fn settings_all(db: tauri::State<'_, Db>) -> Result<HashMap<String, String>, String> {
    db.all_settings()
}

#[tauri::command]
pub fn settings_set(db: tauri::State<'_, Db>, key: String, value: String) -> Result<(), String> {
    db.set_setting(&key, &value)
}

#[tauri::command]
pub fn memories_list(db: tauri::State<'_, Db>, limit: Option<usize>) -> Result<Vec<MemoryRow>, String> {
    db.list_memories(limit.unwrap_or(200))
}

#[tauri::command]
pub fn memory_delete(db: tauri::State<'_, Db>, id: i64) -> Result<(), String> {
    db.delete_memory(id)
}

#[tauri::command]
pub fn memory_report(db: tauri::State<'_, Db>) -> Result<MemoryReport, String> {
    db.memory_report(30)
}

#[tauri::command]
pub fn memory_prune(db: tauri::State<'_, Db>, ids: Vec<i64>) -> Result<usize, String> {
    let mut removed = 0;
    for id in ids {
        db.delete_memory(id)?;
        removed += 1;
    }
    Ok(removed)
}

#[tauri::command]
pub fn search(
    db: tauri::State<'_, Db>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<SearchHit>, String> {
    db.search(&query, limit.unwrap_or(40))
}

#[tauri::command]
pub fn schedules_list(db: tauri::State<'_, Db>) -> Result<Vec<ScheduleRow>, String> {
    db.list_schedules()
}

#[tauri::command]
pub fn schedule_create(
    db: tauri::State<'_, Db>,
    session_id: String,
    prompt: String,
    interval_seconds: i64,
) -> Result<i64, String> {
    db.create_schedule(&session_id, &prompt, interval_seconds)
}

#[tauri::command]
pub fn schedule_delete(db: tauri::State<'_, Db>, id: i64) -> Result<(), String> {
    db.delete_schedule(id)
}

#[tauri::command]
pub fn schedule_toggle(db: tauri::State<'_, Db>, id: i64, enabled: bool) -> Result<(), String> {
    db.set_schedule_enabled(id, enabled)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_history_and_todos() {
        let directory = std::env::temp_dir().join(format!("ds-ide-db-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("test.sqlite3");

        let db = Db::open(&path).unwrap();
        db.ensure_session("s1", "Test", "project", "/tmp").unwrap();
        db.insert_message("s1", "user", Some("hallo"), None, None)
            .unwrap();
        let assistant_id = db
            .insert_message("s1", "assistant", Some("hi"), Some("[]"), None)
            .unwrap();
        db.insert_tool_call(
            "s1",
            Some(assistant_id),
            "call_1",
            "glob",
            Some("{\"pattern\":\"*\"}"),
            Some("ok"),
            false,
            Some(3),
        )
        .unwrap();
        db.replace_todos(
            "s1",
            &[serde_json::json!({ "content": "A", "status": "pending" })],
        )
        .unwrap();

        let history = db.load_history("s1").unwrap();
        assert_eq!(history.len(), 2, "user + assistant");
        assert_eq!(history[0].role, "user");
        assert_eq!(history[1].tools.len(), 1, "tool call attached to assistant");
        assert_eq!(history[1].tools[0].name, "glob");
        assert_eq!(db.list_todos("s1").unwrap().len(), 1);
        assert_eq!(db.list_todos("s1").unwrap()[0].content, "A");

        let llm = db.load_llm_messages("s1").unwrap();
        assert_eq!(llm.len(), 2);
        assert_eq!(llm[1].role, "assistant");

        let sessions = db.list_sessions().unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "s1");

        let _ = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn memories_round_trip() {
        let directory = std::env::temp_dir().join(format!("ds-ide-mem-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("m.sqlite3");

        let db = Db::open(&path).unwrap();
        let id = db
            .insert_memory("preference", "user", "Mag kurze Antworten", None)
            .unwrap();
        assert!(id > 0);

        let list = db.list_memories(10).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].scope, "user");
        assert_eq!(list[0].kind, "preference");

        let context = db.memory_context(10).unwrap();
        assert_eq!(context.len(), 1);
        db.touch_memories(&[id]).unwrap();

        db.delete_memory(id).unwrap();
        assert!(db.list_memories(10).unwrap().is_empty());

        let _ = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn search_finds_sessions_and_messages() {
        let directory =
            std::env::temp_dir().join(format!("ds-ide-search-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        let db = Db::open(&directory.join("s.sqlite3")).unwrap();

        db.ensure_session("s1", "Monaco Editor", "project", "/tmp")
            .unwrap();
        db.insert_message("s1", "user", Some("Wie baue ich Monaco ein?"), None, None)
            .unwrap();
        db.insert_message(
            "s1",
            "assistant",
            Some("Tauri und Vite moegen Worker sehr."),
            None,
            None,
        )
        .unwrap();

        let by_title = db.search("Monaco", 20).unwrap();
        assert!(
            by_title.iter().any(|hit| hit.kind == "session"),
            "title hit expected"
        );

        let by_body = db.search("Worker", 20).unwrap();
        assert!(
            by_body
                .iter()
                .any(|hit| hit.kind == "message" && hit.snippet.contains("Worker")),
            "message hit with snippet expected"
        );

        assert!(db.search("   ", 20).unwrap().is_empty(), "blank query");

        let _ = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn schedules_cycle() {
        let directory =
            std::env::temp_dir().join(format!("ds-ide-sched-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        let db = Db::open(&directory.join("c.sqlite3")).unwrap();

        db.ensure_session("s1", "Test", "project", "/tmp").unwrap();
        let id = db.create_schedule("s1", "Status?", 60).unwrap();

        let first = db.list_schedules().unwrap();
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].interval_seconds, 60);

        // Not yet due …
        assert!(db
            .due_schedules(first[0].next_run_at - 1)
            .unwrap()
            .is_empty());
        // … but due later.
        let due = db.due_schedules(unix_now() + 3600).unwrap();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].prompt, "Status?");

        db.mark_schedule_run(id, unix_now() + 99_999).unwrap();
        assert!(db.due_schedules(unix_now() + 3600).unwrap().is_empty());

        db.set_schedule_enabled(id, false).unwrap();
        assert!(!db.list_schedules().unwrap()[0].enabled);

        db.delete_schedule(id).unwrap();
        assert!(db.list_schedules().unwrap().is_empty());

        let _ = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn memory_report_flags_exact_duplicates() {
        let directory =
            std::env::temp_dir().join(format!("ds-ide-doctor-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        let db = Db::open(&directory.join("d.sqlite3")).unwrap();

        db.insert_memory("preference", "user", "Mag kurze Antworten", None)
            .unwrap();
        db.insert_memory("preference", "user", "mag  kurze   antworten", None)
            .unwrap();
        db.insert_memory("fact", "user", "Etwas anderes", None)
            .unwrap();

        let report = db.memory_report(30).unwrap();
        assert_eq!(report.total, 3);
        assert_eq!(report.duplicates.len(), 1, "one repeat of the first entry");
        assert!(report.stale.is_empty(), "fresh entries are not stale");

        let _ = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn sessions_can_be_renamed_and_pinned() {
        let directory =
            std::env::temp_dir().join(format!("ds-ide-session-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        let db = Db::open(&directory.join("s.sqlite3")).unwrap();

        db.ensure_session("s1", "", "project", "/tmp").unwrap();
        assert_eq!(db.session_title("s1").unwrap().as_deref(), Some(""));

        db.update_session("s1", Some("Monaco-Anbindung"), None)
            .unwrap();
        assert_eq!(
            db.session_title("s1").unwrap().as_deref(),
            Some("Monaco-Anbindung")
        );

        db.update_session("s1", None, Some("pinned")).unwrap();
        assert_eq!(db.list_sessions().unwrap()[0].group, "pinned");

        let _ = std::fs::remove_dir_all(&directory);
    }
}
