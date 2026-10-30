//! Daily housekeeping that costs nothing.
//!
//! Memories rot: the same note gets written twice, a note stops being true, one is never used
//! again. A model could sort that out every night — and pay for a call nobody asked for. This is
//! the cheap half of the same idea: once a day, a deterministic pass counts the duplicates and
//! the stale entries, records the result and says so. It changes nothing. The partner decides
//! what to do with the finding, and the user can act on it in the memory panel.
//!
//! It runs in the same 30-second loop as the scheduler, so it only happens while DS.IDE is
//! running — with the tray icon that is most of the day, which is enough for "once a day".

use tauri::{AppHandle, Manager};

/// Where the last result is kept. A settings key, not a table: one row of history is all the
/// value there is.
const KEY: &str = "memoryCheck";

/// Runs the pass if it has not run today. `force` exists for the button in the panel.
pub fn run(app: &AppHandle, force: bool) -> Option<Value> {
    let db = app.try_state::<crate::db::Db>()?;
    let today = crate::db::local_day();
    if !force {
        let last = db
            .all_settings()
            .ok()
            .and_then(|settings| settings.get(KEY).cloned())
            .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
            .and_then(|value| value.get("day").and_then(Value::as_str).map(str::to_string));
        if last.as_deref() == Some(today.as_str()) {
            return None;
        }
    }

    let report = match db.memory_report(30) {
        Ok(report) => report,
        Err(error) => {
            crate::log_line!("memory check: {error}");
            return None;
        }
    };
    let value = serde_json::json!({
        "day": today,
        "total": report.total,
        "duplicates": report.duplicates.len(),
        "stale": report.stale.len(),
    });
    if let Err(error) = db.set_setting(KEY, &value.to_string()) {
        crate::log_line!("memory check: could not record the result: {error}");
    }
    if report.duplicates.is_empty() && report.stale.is_empty() {
        crate::log_line!("memory check: {} notes, nothing to clean", report.total);
    } else {
        crate::log_line!(
            "memory check: {} notes, {} duplicate(s), {} never used",
            report.total,
            report.duplicates.len(),
            report.stale.len()
        );
        // The panel listens for this and offers the list; nothing is changed from here.
        let _ = app.emit("memory://checked", value.clone());
    }
    Some(value)
}

/// The last result, for the panel's first paint.
#[tauri::command]
pub fn memory_check_status(
    db: tauri::State<'_, crate::db::Db>,
) -> Result<Option<Value>, String> {
    Ok(db
        .all_settings()?
        .get(KEY)
        .and_then(|raw| serde_json::from_str::<Value>(raw).ok()))
}

/// Runs the pass now, whatever the day says. The panel's "check now" button.
#[tauri::command]
pub fn memory_check_now(app: AppHandle) -> Result<Option<Value>, String> {
    Ok(run(&app, true))
}

use serde_json::Value;
use tauri::Emitter;
