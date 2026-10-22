use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

const MAX_LINES: usize = 400;
const MAX_CHARS: usize = 2000;

static LINES: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();

fn buffer() -> &'static Mutex<VecDeque<String>> {
    LINES.get_or_init(|| Mutex::new(VecDeque::new()))
}

/// One line into the in-memory log and into stderr. The ring is deliberately small: the log
/// tab and the "logs" tool are for the last failure, not for keeping a transcript.
pub fn push(line: impl Into<String>) {
    let mut text = line.into();
    if text.chars().count() > MAX_CHARS {
        text = text.chars().take(MAX_CHARS).collect::<String>();
        text.push('…');
    }
    eprintln!("{text}");
    if let Ok(mut lines) = buffer().lock() {
        lines.push_back(text);
        while lines.len() > MAX_LINES {
            lines.pop_front();
        }
    }
}

pub fn tail(limit: usize) -> Vec<String> {
    let limit = limit.clamp(1, MAX_LINES);
    match buffer().lock() {
        Ok(lines) => lines.iter().rev().take(limit).rev().cloned().collect(),
        Err(_) => Vec::new(),
    }
}

#[tauri::command]
pub fn log_tail(limit: Option<usize>) -> Vec<String> {
    tail(limit.unwrap_or(120))
}

#[macro_export]
macro_rules! log_line {
    ($($arg:tt)*) => {{
        $crate::logs::push(format!($($arg)*));
    }};
}
