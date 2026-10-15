use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection};
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
PRAGMA user_version = 1;
";

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
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
}
