use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Mutex;

use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

/// One PTY per terminal tab. The reader thread owns the read handle; the session keeps the
/// writer, the master (for resize) and the child (to stop it when the tab is closed).
#[derive(Default)]
pub struct PtyState {
    sessions: Mutex<HashMap<String, Session>>,
}

struct Session {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn Child + Send + Sync>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PtyOutput {
    id: String,
    data: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PtyExit {
    id: String,
}

fn shell() -> CommandBuilder {
    #[cfg(windows)]
    {
        // PowerShell exists on every supported Windows; cmd is one command away if needed.
        let mut cmd = CommandBuilder::new("powershell.exe");
        cmd.arg("-NoLogo");
        cmd
    }
    #[cfg(not(windows))]
    {
        let program = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        CommandBuilder::new(program)
    }
}

#[tauri::command]
pub fn pty_open(
    app: AppHandle,
    state: State<'_, PtyState>,
    id: String,
    cwd: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    // A tab can be reopened while an old shell is still around; do not leak it.
    let previous = state
        .sessions
        .lock()
        .map_err(|_| "terminal: state poisoned".to_string())?
        .remove(&id);
    if let Some(mut previous) = previous {
        let _ = previous.child.kill();
    }

    let size = PtySize {
        rows: rows.max(2),
        cols: cols.max(10),
        pixel_width: 0,
        pixel_height: 0,
    };
    let pair = native_pty_system()
        .openpty(size)
        .map_err(|error| format!("terminal: {error}"))?;

    let mut command = shell();
    if Path::new(&cwd).is_dir() {
        command.cwd(&cwd);
    } else if let Ok(home) = std::env::current_dir() {
        command.cwd(home);
    }
    command.env("TERM", "xterm-256color");
    command.env("COLORTERM", "truecolor");

    let child = pair
        .slave
        .spawn_command(command)
        .map_err(|error| format!("terminal: {error}"))?;
    drop(pair.slave);

    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|error| format!("terminal: {error}"))?;
    let writer = pair
        .master
        .take_writer()
        .map_err(|error| format!("terminal: {error}"))?;
    let master = pair.master;

    let thread_id = id.clone();
    let thread_app = app.clone();
    std::thread::spawn(move || {
        let mut buffer = [0u8; 8192];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(count) => {
                    let data = String::from_utf8_lossy(&buffer[..count]).to_string();
                    let _ = thread_app.emit(
                        "pty-output",
                        PtyOutput {
                            id: thread_id.clone(),
                            data,
                        },
                    );
                }
                Err(_) => break,
            }
        }
        let _ = thread_app.emit("pty-exit", PtyExit { id: thread_id });
    });

    state
        .sessions
        .lock()
        .map_err(|_| "terminal: state poisoned".to_string())?
        .insert(
            id,
            Session {
                master,
                writer,
                child,
            },
        );
    Ok(())
}

#[tauri::command]
pub fn pty_write(state: State<'_, PtyState>, id: String, data: String) -> Result<(), String> {
    let mut sessions = state
        .sessions
        .lock()
        .map_err(|_| "terminal: state poisoned".to_string())?;
    let session = sessions
        .get_mut(&id)
        .ok_or_else(|| "terminal: not running".to_string())?;
    session
        .writer
        .write_all(data.as_bytes())
        .map_err(|error| format!("terminal: {error}"))?;
    session
        .writer
        .flush()
        .map_err(|error| format!("terminal: {error}"))
}

#[tauri::command]
pub fn pty_resize(
    state: State<'_, PtyState>,
    id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let sessions = state
        .sessions
        .lock()
        .map_err(|_| "terminal: state poisoned".to_string())?;
    let session = sessions
        .get(&id)
        .ok_or_else(|| "terminal: not running".to_string())?;
    session
        .master
        .resize(PtySize {
            rows: rows.max(2),
            cols: cols.max(10),
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|error| format!("terminal: {error}"))
}

#[tauri::command]
pub fn pty_close(state: State<'_, PtyState>, id: String) -> Result<(), String> {
    let session = state
        .sessions
        .lock()
        .map_err(|_| "terminal: state poisoned".to_string())?
        .remove(&id);
    if let Some(mut session) = session {
        let _ = session.child.kill();
    }
    Ok(())
}
