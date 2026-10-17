use serde::Serialize;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

const PREFIX: &str = "browser-";
const MAX_WINDOWS: u32 = 6;

fn parse(url: &str) -> Result<tauri::Url, String> {
    url.parse::<tauri::Url>()
        .map_err(|error| format!("browser: {error}"))
}

fn raise(window: &WebviewWindow) {
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
}

fn open_windows(app: &AppHandle) -> Vec<(u32, WebviewWindow)> {
    (1..=MAX_WINDOWS)
        .filter_map(|index| {
            app.get_webview_window(&format!("{PREFIX}{index}"))
                .map(|window| (index, window))
        })
        .collect()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserWindowInfo {
    pub id: String,
    pub url: String,
}

/// Opens a URL in a window we own.
///
/// `new_window` decides between reusing the most recent window and opening another one —
/// web design work needs several pages side by side, everything else wants one. The cap
/// exists so a stray loop cannot fill the desktop.
///
/// Note: each window carries native chrome instead of a toolbar of ours. Putting our own
/// toolbar above a foreign page needs Tauri's multi-webview support, which is behind the
/// `unstable` feature flag — not something to ship in a product.
#[tauri::command]
pub fn open_browser_window(
    app: AppHandle,
    url: String,
    new_window: Option<bool>,
    focus: Option<bool>,
) -> Result<String, String> {
    let target = parse(&url)?;
    let raise_after = focus.unwrap_or(true);
    let open = open_windows(&app);

    if !new_window.unwrap_or(false) {
        if let Some((_, window)) = open.last() {
            window
                .navigate(target)
                .map_err(|error| format!("browser: {error}"))?;
            if raise_after {
                raise(window);
            }
            return Ok(window.label().to_string());
        }
    }

    let used: Vec<u32> = open.iter().map(|(index, _)| *index).collect();
    let Some(slot) = (1..=MAX_WINDOWS).find(|index| !used.contains(index)) else {
        return Err(format!("browser: {MAX_WINDOWS} windows are already open"));
    };

    let label = format!("{PREFIX}{slot}");
    let window = WebviewWindowBuilder::new(&app, &label, WebviewUrl::External(target))
        .title("DS.IDE — Webdesign")
        .inner_size(1120.0, 840.0)
        .min_inner_size(520.0, 400.0)
        // Following the companion must not take the keyboard away from the composer.
        .focused(raise_after)
        .build()
        .map_err(|error| format!("browser: {error}"))?;
    if raise_after {
        raise(&window);
    }
    Ok(label)
}

#[tauri::command]
pub fn browser_windows(app: AppHandle) -> Vec<BrowserWindowInfo> {
    open_windows(&app)
        .into_iter()
        .map(|(_, window)| BrowserWindowInfo {
            id: window.label().to_string(),
            url: window
                .url()
                .map(|url| url.to_string())
                .unwrap_or_default(),
        })
        .collect()
}

#[tauri::command]
pub fn browser_action(app: AppHandle, id: String, action: String) -> Result<(), String> {
    let Some(window) = app.get_webview_window(&id) else {
        return Err(format!("browser: no window '{id}'"));
    };

    if action == "focus" {
        raise(&window);
        return Ok(());
    }

    let script = match action.as_str() {
        "back" => "history.back()",
        "forward" => "history.forward()",
        "reload" => "location.reload()",
        other => return Err(format!("browser: unknown action '{other}'")),
    };
    window.eval(script).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn browser_close(app: AppHandle, id: String) -> Result<(), String> {
    let Some(window) = app.get_webview_window(&id) else {
        return Err(format!("browser: no window '{id}'"));
    };
    window.close().map_err(|error| error.to_string())
}
