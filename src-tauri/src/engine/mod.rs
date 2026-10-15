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
            match db.insert_memory(kind, scope, text, None) {
                Ok(id) => tools::ToolOutcome::ok(format!("remembered as #{id} ({scope}/{kind})")),
                Err(error) => tools::ToolOutcome::err(format!("memory: {error}")),
            }
        }
        "forget" => {
            let Some(id) = args.get("id").and_then(Value::as_i64) else {
                return tools::ToolOutcome::err("memory: 'id' is required for forget");
            };
            match db.delete_memory(id) {
                Ok(()) => tools::ToolOutcome::ok(format!("forgot #{id}")),
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
                    .map(|row| format!("#{} [{}/{}] {}", row.id, row.scope, row.kind, row.text))
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
            model: std::env::var("DS_IDE_MODEL").unwrap_or_else(|_| "deepseek-chat".to_string()),
            system_prompt: String::new(),
            workspace: std::env::temp_dir().to_string_lossy().to_string(),
            full_authorization: true,
            max_turns: 4,
        })
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
    let definitions = tools::definitions();
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
                system_prompt.push_str("\n\n## What you already know\n");
                for row in &recall {
                    system_prompt.push_str(&format!("- [{}] {}\n", row.scope, row.text));
                }
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
