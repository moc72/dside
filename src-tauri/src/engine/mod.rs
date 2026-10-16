pub mod deepseek;
pub mod tools;
pub mod types;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::oneshot;

use crate::db::Db;
use crate::secret::SecretStore;
use types::{ActionEvent, ChatMessage, EngineConfig, EngineEvent, ToolEvent, UsageEvent};

const EVENT_CHANNEL: &str = "engine://event";

const DEFAULT_SYSTEM_PROMPT: &str = "You are the coding partner inside DS.IDE, a desktop agentic IDE. \
You have full tool access: read, write, edit, bash, glob, grep, todowrite. \
Work directly without unnecessary confirmation, do exactly the work that was asked, report faithfully \
what succeeded, what failed and what is unverified, and answer in the user's language.";

#[derive(Default)]
pub struct SessionState {
    cancel: Option<Arc<AtomicBool>>,
    seq: u64,
}

#[derive(Default)]
pub struct EngineState {
    sessions: Mutex<HashMap<String, SessionState>>,
    config: Mutex<Option<EngineConfig>>,
    /// Questions a running turn is blocked on. The turn awaits the receiver; the
    /// `engine_answer` command feeds it.
    pending: Mutex<HashMap<String, oneshot::Sender<String>>>,
}

fn emit(app: &AppHandle, event: EngineEvent) {
    let _ = app.emit(EVENT_CHANNEL, event);
}

#[tauri::command]
pub fn engine_set_config(state: State<'_, EngineState>, config: EngineConfig) -> Result<(), String> {
    let mut slot = state.config.lock().map_err(|error| error.to_string())?;
    *slot = Some(config);
    Ok(())
}

#[tauri::command]
pub fn engine_configured(app: AppHandle) -> bool {
    app.try_state::<SecretStore>()
        .map(|store| store.status().present)
        .unwrap_or(false)
}

#[tauri::command]
pub fn engine_reset(app: AppHandle, session_id: String) -> Result<(), String> {
    if let Some(db) = app.try_state::<Db>() {
        db.delete_session(&session_id)?;
        db.create_session(&session_id, "", "")?;
    }
    let state = app.state::<EngineState>();
    if let Ok(mut sessions) = state.sessions.lock() {
        sessions.remove(&session_id);
    }
    Ok(())
}

#[tauri::command]
pub fn engine_abort(app: AppHandle, session_id: String) -> Result<(), String> {
    let state = app.state::<EngineState>();
    let sessions = state.sessions.lock().map_err(|error| error.to_string())?;
    if let Some(session) = sessions.get(&session_id) {
        if let Some(flag) = &session.cancel {
            flag.store(true, Ordering::SeqCst);
        }
    }
    Ok(())
}

#[tauri::command]
pub fn engine_send(app: AppHandle, session_id: String, prompt: String) -> Result<(), String> {
    send_prompt(&app, &session_id, &prompt)
}

/// What the provider reports for one model. The API knows friendlier names than the id,
/// so the UI shows those and sends the id.
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub context_window: Option<i64>,
    /// The provider's thinking levels for this model, as it reports them. Empty when the
    /// model does not think, and then we do not offer a choice.
    pub efforts: Vec<String>,
    pub default_effort: Option<String>,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceInfo {
    pub available: bool,
    pub currency: String,
    pub total: String,
    pub granted: String,
    pub topped_up: String,
}

async fn provider_request(app: &AppHandle, path: &str) -> Result<Value, String> {
    let base = {
        let state = app.state::<EngineState>();
        let slot = state.config.lock().map_err(|error| error.to_string())?;
        slot.as_ref()
            .map(|config| config.base_url.clone())
            .unwrap_or_else(|| "https://api.deepseek.com".to_string())
    };
    let key = app
        .try_state::<SecretStore>()
        .and_then(|store| store.get())
        .unwrap_or_default();
    if key.trim().is_empty() {
        return Err("API key is missing".to_string());
    }

    let url = format!("{}/{}", base.trim_end_matches('/'), path.trim_start_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|error| error.to_string())?;

    let response = client
        .get(url)
        .bearer_auth(key)
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }
    response.json::<Value>().await.map_err(|error| error.to_string())
}

/// Asks the provider which models the stored key may use (`GET /models`).
#[tauri::command]
pub async fn engine_models(app: AppHandle) -> Result<Vec<ModelInfo>, String> {
    let body = provider_request(&app, "/models")
        .await
        .map_err(|error| format!("models: {error}"))?;

    let models: Vec<ModelInfo> = body
        .get("data")
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter_map(|entry| {
                    let id = entry.get("id").and_then(Value::as_str)?;
                    let effort = entry.get("effort");
                    Some(ModelInfo {
                        id: id.to_string(),
                        name: entry
                            .get("name")
                            .and_then(Value::as_str)
                            .unwrap_or(id)
                            .to_string(),
                        context_window: entry.get("context_window").and_then(Value::as_i64),
                        efforts: effort
                            .and_then(|value| value.get("supported_levels"))
                            .and_then(Value::as_array)
                            .map(|levels| {
                                levels
                                    .iter()
                                    .filter_map(Value::as_str)
                                    .map(str::to_string)
                                    .collect()
                            })
                            .unwrap_or_default(),
                        default_effort: effort
                            .and_then(|value| value.get("default_level"))
                            .and_then(Value::as_str)
                            .map(str::to_string),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    if models.is_empty() {
        return Err("models: the provider returned no model list".to_string());
    }
    Ok(models)
}

/// Credits left, straight from the provider (`GET /user/balance`).
#[tauri::command]
pub async fn provider_balance(app: AppHandle) -> Result<BalanceInfo, String> {
    let body = provider_request(&app, "/user/balance")
        .await
        .map_err(|error| format!("balance: {error}"))?;
    let first = body
        .get("balance_infos")
        .and_then(Value::as_array)
        .and_then(|list| list.first());

    let text = |entry: Option<&Value>, key: &str| {
        entry
            .and_then(|value| value.get(key))
            .and_then(Value::as_str)
            .unwrap_or("0")
            .to_string()
    };

    Ok(BalanceInfo {
        available: body
            .get("is_available")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        currency: text(first, "currency"),
        total: text(first, "total_balance"),
        granted: text(first, "granted_balance"),
        topped_up: text(first, "topped_up_balance"),
    })
}

/// Resolves a pending `ACTION_REQUIRED` — the turn that asked continues with this answer.
#[tauri::command]
pub fn engine_answer(app: AppHandle, action_id: String, answer: String) -> Result<(), String> {
    let state = app.state::<EngineState>();
    let sender = state
        .pending
        .lock()
        .map_err(|error| error.to_string())?
        .remove(&action_id);
    match sender {
        Some(sender) => {
            let _ = sender.send(answer);
            Ok(())
        }
        None => Err("engine: no pending question with that id".to_string()),
    }
}

/// Starts one turn. Shared by the `engine_send` command and the scheduler, so a timed run
/// goes through exactly the same path (persistence, tools, events) as a typed one.
pub fn send_prompt(app: &AppHandle, session_id: &str, prompt: &str) -> Result<(), String> {
    let mut config = {
        let state = app.state::<EngineState>();
        let slot = state.config.lock().map_err(|error| error.to_string())?;
        slot.clone()
            .ok_or_else(|| "engine: not configured".to_string())?
    };

    // The key never crosses the webview boundary: it is read here, per turn.
    config.api_key = app
        .try_state::<SecretStore>()
        .and_then(|store| store.get())
        .unwrap_or_default();
    if config.api_key.trim().is_empty() {
        return Err("engine: API key is missing".to_string());
    }

    // Persist the user turn first: the conversation must survive a crash or restart.
    let messages = match app.try_state::<Db>() {
        Some(db) => {
            db.ensure_session(session_id, "", "project", &config.workspace)?;
            db.insert_message(session_id, "user", Some(prompt), None, None)?;
            db.touch_session(session_id)?;
            db.load_llm_messages(session_id)?
        }
        None => vec![ChatMessage::user(prompt.to_string())],
    };

    let cancel = Arc::new(AtomicBool::new(false));
    let seq = {
        let state = app.state::<EngineState>();
        let mut sessions = state.sessions.lock().map_err(|error| error.to_string())?;
        let session = sessions.entry(session_id.to_string()).or_default();
        session.cancel = Some(cancel.clone());
        session.seq
    };

    // A brand-new session is named after its first message.
    maybe_generate_title(app, session_id, prompt, config.clone());

    let app_handle = app.clone();
    let sid = session_id.to_string();

    tauri::async_runtime::spawn(async move {
        let (seq, outcome) = run_turn(&app_handle, &config, &sid, messages, seq, cancel).await;

        {
            let state = app_handle.state::<EngineState>();
            if let Ok(mut sessions) = state.sessions.lock() {
                let session = sessions.entry(sid.clone()).or_default();
                session.seq = seq;
                session.cancel = None;
            }
        }

        if let Err(error) = outcome {
            let mut event = EngineEvent::new(&sid, seq + 1, "error");
            event.error = Some(error);
            emit(&app_handle, event);

            let mut terminal = EngineEvent::new(&sid, seq + 2, "turn_end");
            terminal.stop_reason = Some("error".to_string());
            emit(&app_handle, terminal);
        }
    });

    Ok(())
}

/// Called from a background loop: start every schedule that is due.
pub fn run_due_schedules(app: &AppHandle) {
    let Some(db) = app.try_state::<Db>() else {
        return;
    };
    let now = crate::db::unix_now();
    let due = match db.due_schedules(now) {
        Ok(rows) => rows,
        Err(error) => {
            eprintln!("scheduler: could not load schedules: {error}");
            return;
        }
    };

    for schedule in due {
        let _ = db.mark_schedule_run(schedule.id, now + schedule.interval_seconds);
        if let Err(error) = send_prompt(app, &schedule.session_id, &schedule.prompt) {
            eprintln!("scheduler: schedule {} failed: {error}", schedule.id);
        }
    }
}

/// The part of the system prompt that hands the partner its own memory. Split out so the
/// wording — scopes and the user's marker included — is testable without a live turn.
fn recall_block(rows: &[crate::db::MemoryRow]) -> String {
    if rows.is_empty() {
        return String::new();
    }

    let mut block = String::from(
        "\n\n## Your long-term memory\n\
         These are notes you kept from earlier work, and things you know about the user.\n\
         - `[you]` — your own note. `[user]` — what you know about the user. `[session]` — tied to one task.\n\
         - `(changed by the user)` means the user rewrote that note themselves: treat the wording\n\
         as theirs, not as something you inferred.\n\
         You are this person's partner, so use these notes the way a colleague would use what\n\
         they remember: silently, and without reciting them back.\n",
    );

    for row in rows {
        let scope = match row.scope.as_str() {
            "agent" => "you",
            other => other,
        };
        let marker = if row.origin == "user" {
            " (changed by the user)"
        } else {
            ""
        };
        block.push_str(&format!("- [{scope}] {}{marker}\n", row.text));
    }
    block
}

/// The partner's own name: chosen once, by the model itself, and never changed again.
fn choose_name_tool(app: &AppHandle, args: &Value) -> tools::ToolOutcome {
    let Some(db) = app.try_state::<Db>() else {
        return tools::ToolOutcome::err("choose_name: database unavailable");
    };
    let name = args
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    if name.is_empty() {
        return tools::ToolOutcome::err("choose_name: 'name' is required");
    }

    match db.set_partner_name(&name) {
        Ok(()) => tools::ToolOutcome::ok(format!(
            "From now on your name is {name}. The user sees it in the settings; use it for yourself."
        )),
        Err(error) => tools::ToolOutcome::err(format!("choose_name: {error}")),
    }
}

/// Long-term memory lives in the database, so it is handled here rather than in
/// `tools::execute`, which stays storage-free.
fn memory_tool(app: &AppHandle, args: &Value) -> tools::ToolOutcome {
    let Some(db) = app.try_state::<Db>() else {
        return tools::ToolOutcome::err("memory: database unavailable");
    };

    let action = args.get("action").and_then(Value::as_str).unwrap_or("list");
    match action {
        "add" => {
            let text = args
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim();
            if text.is_empty() {
                return tools::ToolOutcome::err("memory: 'text' is required for add");
            }
            let kind = args.get("kind").and_then(Value::as_str).unwrap_or("fact");
            let scope = args.get("scope").and_then(Value::as_str).unwrap_or("user");
            // Seeky's own note. Entries the user rewrote in the UI carry origin 'user'.
            match db.insert_memory(kind, scope, text, None, "seeky") {
                Ok(id) => tools::ToolOutcome::ok(format!("remembered as #{id} ({scope}/{kind})")),
                Err(error) => tools::ToolOutcome::err(format!("memory: {error}")),
            }
        }
        "update" => {
            let Some(id) = args.get("id").and_then(Value::as_i64) else {
                return tools::ToolOutcome::err("memory: 'id' is required for update");
            };
            let text = args
                .get("text")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty());
            let kind = args.get("kind").and_then(Value::as_str);
            let importance = args.get("importance").and_then(Value::as_i64);
            if text.is_none() && kind.is_none() && importance.is_none() {
                return tools::ToolOutcome::err("memory: nothing to update");
            }
            // Seeky rewrote it itself, so the entry is its own again — the user marker is
            // only for changes the user made.
            match db.update_memory(id, text, kind, importance, "seeky") {
                Ok(()) => tools::ToolOutcome::ok(format!("updated #{id}")),
                Err(error) => tools::ToolOutcome::err(format!("memory: {error}")),
            }
        }
        "forget" => {
            let Some(id) = args.get("id").and_then(Value::as_i64) else {
                return tools::ToolOutcome::err("memory: 'id' is required for forget");
            };
            match db.retire_memory(id) {
                Ok(()) => tools::ToolOutcome::ok(format!(
                    "set #{id} aside — it stays in the database but is no longer recalled"
                )),
                Err(error) => tools::ToolOutcome::err(format!("memory: {error}")),
            }
        }
        "curate" => match db.memory_report(30) {
            Ok(report) => tools::ToolOutcome::ok(format!(
                "{} memories; {} exact duplicate(s) {:?}; {} stale and never used {:?}",
                report.total,
                report.duplicates.len(),
                report.duplicates,
                report.stale.len(),
                report.stale
            )),
            Err(error) => tools::ToolOutcome::err(format!("memory: {error}")),
        },
        _ => match db.list_memories(50) {
            Ok(rows) if rows.is_empty() => tools::ToolOutcome::ok("(no memories stored)"),
            Ok(rows) => {
                let body = rows
                    .iter()
                    .map(|row| {
                        let marker = if row.origin == "user" {
                            " (changed by the user)"
                        } else {
                            ""
                        };
                        format!("#{} [{}/{}] {}{marker}", row.id, row.scope, row.kind, row.text)
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                tools::ToolOutcome::ok(body)
            }
            Err(error) => tools::ToolOutcome::err(format!("memory: {error}")),
        },
    }
}

/// Blocks the turn until the user answers. This is the real `ACTION_REQUIRED` round-trip:
/// emit the question, park a sender, await the receiver.
async fn ask_user(
    app: &AppHandle,
    session_id: &str,
    args: &Value,
    seq: &mut u64,
) -> tools::ToolOutcome {
    let question = args
        .get("question")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    if question.is_empty() {
        return tools::ToolOutcome::err("ask_user: 'question' is required");
    }
    let options: Vec<String> = args
        .get("options")
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter_map(|value| value.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let action_id = format!("q{}", crate::db::unix_now());
    let (sender, receiver) = oneshot::channel::<String>();
    {
        let state = app.state::<EngineState>();
        if let Ok(mut pending) = state.pending.lock() {
            pending.insert(action_id.clone(), sender);
        }
    }

    *seq += 1;
    let mut event = EngineEvent::new(session_id, *seq, "action_required");
    event.action = Some(ActionEvent {
        id: action_id.clone(),
        kind: "question".to_string(),
        question,
        options,
    });
    emit(app, event);

    match tokio::time::timeout(std::time::Duration::from_secs(300), receiver).await {
        Ok(Ok(answer)) => tools::ToolOutcome::ok(answer),
        Ok(Err(_)) => tools::ToolOutcome::err("ask_user: the question was dropped"),
        Err(_) => tools::ToolOutcome::err("ask_user: no answer within 5 minutes"),
    }
}

/// A sub-agent: a fresh conversation with the same model and tools, returning only its
/// final answer. Its work is deliberately invisible in the UI — only the tool card shows —
/// so the point is to keep the parent context clean.
///
/// Guards: no nested `task` (no recursion), no `ask_user` (it could not reach the user
/// through the parent's UI), and a hard inner turn cap.
pub async fn task_tool(config: &EngineConfig, args: &Value) -> tools::ToolOutcome {
    const MAX_INNER_TURNS: u32 = 8;
    const MAX_ANSWER_CHARS: usize = 20_000;

    let prompt = args
        .get("prompt")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let Some(prompt) = prompt else {
        return tools::ToolOutcome::err("task: 'prompt' is required");
    };
    if config.api_key.trim().is_empty() {
        return tools::ToolOutcome::err("task: engine is not configured");
    }

    let workspace = if config.workspace.trim().is_empty() {
        std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
    } else {
        config.workspace.clone()
    };
    let ctx = tools::ToolContext {
        workspace: PathBuf::from(&workspace),
        full_authorization: config.full_authorization,
        skills_root: crate::skills::skills_root(),
    };

    let definitions: Vec<Value> = tools::definitions()
        .into_iter()
        .filter(|tool| {
            let name = tool
                .pointer("/function/name")
                .and_then(Value::as_str)
                .unwrap_or("");
            name != "task" && name != "ask_user"
        })
        .collect();

    let mut messages = vec![
        ChatMessage::system(
            "You are a sub-agent inside DS.IDE, working on one self-contained task. \
             You have tools (read, write, edit, bash, glob, grep, web_fetch, skill). \
             Do the work and answer with the result and the evidence needed to trust it. \
             Do not ask questions — nobody can answer them here.",
        ),
        ChatMessage::user(prompt.to_string()),
    ];
    let mut todos: Vec<Value> = Vec::new();

    for _ in 0..MAX_INNER_TURNS {
        let mut sink = |_kind: deepseek::DeltaKind, _chunk: &str| {};
        let outcome = match deepseek::stream_chat(config, &messages, &definitions, &mut sink).await {
            Ok(outcome) => outcome,
            Err(error) => return tools::ToolOutcome::err(format!("task: {error}")),
        };

        let has_tool_calls = !outcome.tool_calls.is_empty();
        messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: if outcome.content.is_empty() {
                None
            } else {
                Some(outcome.content.clone())
            },
            tool_calls: if has_tool_calls {
                Some(outcome.tool_calls.clone())
            } else {
                None
            },
            tool_call_id: None,
        });

        if !has_tool_calls {
            let answer = outcome.content.trim();
            if answer.is_empty() {
                return tools::ToolOutcome::ok("(the sub-agent returned no answer)");
            }
            let capped: String = answer.chars().take(MAX_ANSWER_CHARS).collect();
            return tools::ToolOutcome::ok(capped);
        }

        for call in &outcome.tool_calls {
            let arguments: Value =
                serde_json::from_str(&call.function.arguments).unwrap_or(Value::Null);
            let result = tools::execute(&call.function.name, &arguments, &ctx, &mut todos).await;
            messages.push(ChatMessage::tool(call.id.clone(), result.content));
        }
    }

    tools::ToolOutcome::err(format!(
        "task: the sub-agent used its {MAX_INNER_TURNS} turns without a final answer"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn live_config() -> Option<EngineConfig> {
        let api_key = std::env::var("DS_IDE_API_KEY").ok()?;
        if api_key.trim().is_empty() {
            return None;
        }
        Some(EngineConfig {
            api_key,
            base_url: std::env::var("DS_IDE_BASE_URL")
                .unwrap_or_else(|_| "https://api.deepseek.com".to_string()),
            model: std::env::var("DS_IDE_MODEL").unwrap_or_else(|_| "deepseek-flash".to_string()),
            system_prompt: String::new(),
            workspace: std::env::temp_dir().to_string_lossy().to_string(),
            full_authorization: true,
            max_turns: 4,
            reasoning_effort: None,
        })
    }

    /// The recall has to say whose words these are — that marker is the whole point of the
    /// companion rows, and it is easy to lose in a prompt refactor.
    #[test]
    fn recall_marks_scope_and_user_edits() {
        let row = |id: i64, scope: &str, text: &str, origin: &str| crate::db::MemoryRow {
            id,
            kind: "fact".to_string(),
            scope: scope.to_string(),
            text: text.to_string(),
            importance: 1,
            created_at: 0,
            origin: origin.to_string(),
            edited_at: None,
            retired_at: None,
        };

        assert!(recall_block(&[]).is_empty(), "no memory, no block");

        let block = recall_block(&[
            row(1, "user", "Arbeitet an DS.IDE", "seeky"),
            row(2, "agent", "Ich halte Antworten kurz", "seeky"),
            row(3, "user", "Mag kurze Antworten", "user"),
            row(4, "session", "Testet gerade die Vorschau", "seeky"),
        ]);

        assert!(block.contains("## Your long-term memory"));
        assert!(block.contains("- [user] Arbeitet an DS.IDE\n"));
        assert!(
            block.contains("- [you] Ich halte Antworten kurz\n"),
            "an agent-scope note is the partner's own: {block}"
        );
        assert!(
            block.contains("- [user] Mag kurze Antworten (changed by the user)\n"),
            "a user edit is marked: {block}"
        );
        assert!(
            !block.contains("Arbeitet an DS.IDE (changed by the user)"),
            "Seeky's own note stays unmarked"
        );
        assert!(block.contains("- [session] Testet gerade die Vorschau\n"));
    }

    #[tokio::test]
    #[ignore = "requires DS_IDE_API_KEY"]
    async fn live_task_subagent_answers_without_tools() {
        let Some(config) = live_config() else {
            eprintln!("skipped: DS_IDE_API_KEY not set");
            return;
        };
        let outcome = task_tool(
            &config,
            &serde_json::json!({ "prompt": "Antworte mit genau einem Wort: BEREIT" }),
        )
        .await;

        assert!(!outcome.is_error, "task failed: {}", outcome.content);
        assert!(
            outcome.content.to_uppercase().contains("BEREIT"),
            "unexpected answer: {}",
            outcome.content
        );
        println!("subagent answer: {}", outcome.content.trim());
    }

    #[tokio::test]
    #[ignore = "requires DS_IDE_API_KEY"]
    async fn live_task_subagent_uses_its_tools() {
        let Some(config) = live_config() else {
            return;
        };
        let outcome = task_tool(
            &config,
            &serde_json::json!({
                "prompt": "Zähle mit dem Tool glob (pattern '*') die Einträge im aktuellen Verzeichnis und nenne mir die Anzahl als Zahl."
            }),
        )
        .await;

        assert!(!outcome.is_error, "task failed: {}", outcome.content);
        assert!(!outcome.content.trim().is_empty(), "empty sub-agent answer");
        println!("subagent answer: {}", outcome.content.trim());
    }
}

fn emit_title(app: &AppHandle, session_id: &str, title: &str) {
    let mut event = EngineEvent::new(session_id, 0, "title");
    event.text = Some(title.to_string());
    emit(app, event);
}

/// Names a new session after its first message: instantly from the text, then — if the
/// model answers — with a short, human title on top. The instant one means the rail is
/// never empty; the model one is what the user actually reads most of the time.
fn maybe_generate_title(
    app: &AppHandle,
    session_id: &str,
    first_message: &str,
    config: EngineConfig,
) {
    let Some(db) = app.try_state::<Db>() else {
        return;
    };
    let needs_title = db
        .session_title(session_id)
        .ok()
        .flatten()
        .map(|title| title.trim().is_empty())
        .unwrap_or(false);
    if !needs_title {
        return;
    }

    let fallback: String = first_message
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .chars()
        .take(60)
        .collect();
    if fallback.is_empty() {
        return;
    }
    let _ = db.update_session(session_id, Some(&fallback), None);
    emit_title(app, session_id, &fallback);

    let app_handle = app.clone();
    let sid = session_id.to_string();
    let owned_message = first_message.to_string();
    tauri::async_runtime::spawn(async move {
        let messages = vec![
            ChatMessage::system(
                "You name chat sessions. Answer with the title only: three to six words, \
                 no quotes, no trailing period, in the language of the message.",
            ),
            ChatMessage::user(owned_message),
        ];
        let mut sink = |_kind: deepseek::DeltaKind, _chunk: &str| {};
        let Ok(outcome) = deepseek::stream_chat(&config, &messages, &[], &mut sink).await else {
            return;
        };

        let title: String = outcome
            .content
            .trim()
            .trim_matches('"')
            .trim()
            .chars()
            .take(80)
            .collect();
        if title.is_empty() {
            return;
        }
        if let Some(db) = app_handle.try_state::<Db>() {
            let _ = db.update_session(&sid, Some(&title), None);
        }
        emit_title(&app_handle, &sid, &title);
    });
}

async fn run_turn(
    app: &AppHandle,
    config: &EngineConfig,
    session_id: &str,
    mut messages: Vec<ChatMessage>,
    mut seq: u64,
    cancel: Arc<AtomicBool>,
) -> (u64, Result<(), String>) {
    let workspace = if config.workspace.trim().is_empty() {
        std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
    } else {
        config.workspace.clone()
    };
    let ctx = tools::ToolContext {
        workspace: PathBuf::from(&workspace),
        full_authorization: config.full_authorization,
        skills_root: crate::skills::skills_root(),
    };
    // The name can only be chosen once, so the tool disappears from the list afterwards —
    // no point tempting the model with a door that is already closed.
    let named = app
        .try_state::<Db>()
        .and_then(|db| db.partner_name().ok().flatten())
        .is_some();
    let definitions: Vec<Value> = tools::definitions()
        .into_iter()
        .filter(|definition| {
            let is_name_tool = definition
                .get("function")
                .and_then(|function| function.get("name"))
                .and_then(Value::as_str)
                == Some("choose_name");
            named || !is_name_tool
        })
        .collect();
    let mut todos: Vec<Value> = Vec::new();

    let mut system_prompt = if config.system_prompt.trim().is_empty() {
        DEFAULT_SYSTEM_PROMPT.to_string()
    } else {
        config.system_prompt.clone()
    };

    // Recall: hand the partner what it already knows. This is the difference between
    // a persona and a personality — see PROGRESS.md section 12.
    if let Some(db) = app.try_state::<Db>() {
        if let Ok(recall) = db.memory_context(40) {
            if !recall.is_empty() {
                system_prompt.push_str(&recall_block(&recall));
                let ids: Vec<i64> = recall.iter().map(|row| row.id).collect();
                let _ = db.touch_memories(&ids);
            }
        }
    }

    messages.insert(0, ChatMessage::system(system_prompt));

    let max_turns = config.max_turns.clamp(1, 100);
    let mut outcome_result: Result<(), String> = Ok(());

    for _turn in 0..max_turns {
        if cancel.load(Ordering::SeqCst) {
            seq += 1;
            let mut event = EngineEvent::new(session_id, seq, "turn_end");
            event.stop_reason = Some("abort".to_string());
            emit(app, event);
            return (seq, Ok(()));
        }

        seq += 1;
        let mut running = EngineEvent::new(session_id, seq, "status");
        running.status = Some("running".to_string());
        emit(app, running);

        let mut streamed_seq = seq;
        let stream_result = {
            let mut on_delta = |kind: deepseek::DeltaKind, chunk: &str| {
                streamed_seq += 1;
                let mut event = EngineEvent::new(
                    session_id,
                    streamed_seq,
                    match kind {
                        deepseek::DeltaKind::Text => "stream",
                        deepseek::DeltaKind::Reasoning => "reasoning",
                    },
                );
                event.text = Some(chunk.to_string());
                emit(app, event);
            };
            deepseek::stream_chat(config, &messages, &definitions, &mut on_delta).await
        };
        seq = streamed_seq;

        let outcome = match stream_result {
            Ok(outcome) => outcome,
            Err(error) => {
                outcome_result = Err(error);
                break;
            }
        };

        let has_tool_calls = !outcome.tool_calls.is_empty();
        let tool_calls_json = if has_tool_calls {
            serde_json::to_string(&outcome.tool_calls).ok()
        } else {
            None
        };
        let content = if outcome.content.is_empty() {
            None
        } else {
            Some(outcome.content.clone())
        };

        let message_id = app.try_state::<Db>().and_then(|db| {
            db.insert_message(
                session_id,
                "assistant",
                content.as_deref(),
                tool_calls_json.as_deref(),
                None,
            )
            .ok()
        });

        messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: outcome.content.clone().into(),
            tool_calls: if has_tool_calls {
                Some(outcome.tool_calls.clone())
            } else {
                None
            },
            tool_call_id: None,
        });

        if outcome.total_tokens > 0 {
            seq += 1;
            let mut event = EngineEvent::new(session_id, seq, "usage");
            event.usage = Some(UsageEvent {
                prompt_tokens: outcome.prompt_tokens,
                completion_tokens: outcome.completion_tokens,
                total_tokens: outcome.total_tokens,
            });
            emit(app, event);
        }

        if !has_tool_calls {
            seq += 1;
            let mut event = EngineEvent::new(session_id, seq, "turn_end");
            event.stop_reason = Some("end_turn".to_string());
            emit(app, event);
            return (seq, outcome_result);
        }

        for call in &outcome.tool_calls {
            if cancel.load(Ordering::SeqCst) {
                break;
            }
            let arguments: Value =
                serde_json::from_str(&call.function.arguments).unwrap_or(Value::Null);

            seq += 1;
            let mut start = EngineEvent::new(session_id, seq, "tool_start");
            start.tool = Some(ToolEvent {
                id: call.id.clone(),
                name: call.function.name.clone(),
                arguments: Some(call.function.arguments.clone()),
                output: None,
                is_error: None,
                duration_ms: None,
            });
            emit(app, start);

            let started = std::time::Instant::now();
            let tool_outcome = if call.function.name == "memory" {
                memory_tool(app, &arguments)
            } else if call.function.name == "choose_name" {
                choose_name_tool(app, &arguments)
            } else if call.function.name == "ask_user" {
                ask_user(app, session_id, &arguments, &mut seq).await
            } else if call.function.name == "task" {
                task_tool(config, &arguments).await
            } else {
                tools::execute(&call.function.name, &arguments, &ctx, &mut todos).await
            };
            let duration_ms = started.elapsed().as_millis() as i64;

            seq += 1;
            let mut end = EngineEvent::new(session_id, seq, "tool_result");
            end.tool = Some(ToolEvent {
                id: call.id.clone(),
                name: call.function.name.clone(),
                arguments: None,
                output: Some(tool_outcome.content.clone()),
                is_error: Some(tool_outcome.is_error),
                duration_ms: Some(duration_ms as u64),
            });
            emit(app, end);

            if let Some(db) = app.try_state::<Db>() {
                let _ = db.insert_tool_call(
                    session_id,
                    message_id,
                    &call.id,
                    &call.function.name,
                    Some(&call.function.arguments),
                    Some(&tool_outcome.content),
                    tool_outcome.is_error,
                    Some(duration_ms),
                );
                let _ = db.insert_message(
                    session_id,
                    "tool",
                    Some(&tool_outcome.content),
                    None,
                    Some(&call.id),
                );
                if call.function.name == "todowrite" {
                    let _ = db.replace_todos(session_id, &todos);
                }
            }

            messages.push(ChatMessage::tool(call.id.clone(), tool_outcome.content));
        }
    }

    seq += 1;
    let mut event = EngineEvent::new(session_id, seq, "turn_end");
    event.stop_reason = Some(if outcome_result.is_err() {
        "error".to_string()
    } else {
        "max_turns".to_string()
    });
    emit(app, event);

    if let Some(db) = app.try_state::<Db>() {
        let _ = db.touch_session(session_id);
    }

    (seq, outcome_result)
}
