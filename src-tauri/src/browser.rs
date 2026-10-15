use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

const LABEL: &str = "browser";

fn parse(url: &str) -> Result<tauri::Url, String> {
    url.parse::<tauri::Url>()
        .map_err(|error| format!("browser: {error}"))
}

fn raise(window: &tauri::WebviewWindow) {
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
}

/// Opens a URL in a window we own. If it already exists we navigate it and raise it,
/// instead of stacking a second one — that is the whole point of not handing the URL to
/// the user's browser: focus stays ours, so a minimised page can be brought back.
#[tauri::command]
pub fn open_browser_window(app: AppHandle, url: String) -> Result<(), String> {
    let target = parse(&url)?;

    if let Some(window) = app.get_webview_window(LABEL) {
        window
            .navigate(target)
            .map_err(|error| format!("browser: {error}"))?;
        raise(&window);
        return Ok(());
    }

    let window = WebviewWindowBuilder::new(&app, LABEL, WebviewUrl::External(target))
        .title("DS.IDE — Browser")
        .inner_size(1120.0, 840.0)
        .min_inner_size(520.0, 400.0)
        .build()
        .map_err(|error| format!("browser: {error}"))?;
    raise(&window);
    Ok(())
}

#[tauri::command]
pub fn browser_navigate(app: AppHandle, url: String) -> Result<(), String> {
    let target = parse(&url)?;
    let Some(window) = app.get_webview_window(LABEL) else {
        return Err("browser: no window open".to_string());
    };
    window
        .navigate(target)
        .map_err(|error| format!("browser: {error}"))
}

#[tauri::command]
pub fn browser_action(app: AppHandle, action: String) -> Result<(), String> {
    let Some(window) = app.get_webview_window(LABEL) else {
        return Err("browser: no window open".to_string());
    };
    let script = match action.as_str() {
        "back" => "history.back()",
        "forward" => "history.forward()",
        "reload" => "location.reload()",
        "focus" => {
            raise(&window);
            return Ok(());
        }
        other => return Err(format!("browser: unknown action '{other}'")),
    };
    window.eval(script).map_err(|error| error.to_string())
}
