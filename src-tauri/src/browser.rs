use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tokio::time::sleep;

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


/// What the page extraction script writes into the URL hash; the host reads it back without
/// giving the remote page any IPC access.
const READ_SCRIPT: &str = r##"
(() => {
  try {
    const cap = (value, limit) => (value || "").slice(0, limit);
    const links = Array.from(document.querySelectorAll("a[href]"))
      .slice(0, 40)
      .map((a) => ({ text: cap((a.innerText || "").trim(), 120), href: a.href }));
    const fields = Array.from(document.querySelectorAll("input, textarea, select"))
      .slice(0, 60)
      .map((el, index) => {
        const type = (el.type || el.tagName).toLowerCase();
        el.setAttribute("data-dside-field", String(index));
        return {
          selector: '[data-dside-field="' + index + '"]',
          name: el.name || "",
          id: el.id || "",
          type,
          value: type === "password" ? "" : cap(el.value || "", 500),
          placeholder: cap(el.placeholder || "", 160),
          checked: !!el.checked,
          options: el.tagName === "SELECT"
            ? Array.from(el.options).slice(0, 20).map((o) => ({ value: o.value, text: cap((o.text || "").trim(), 80) }))
            : []
        };
      });
    const data = {
      url: location.href,
      title: document.title,
      text: cap(document.body ? document.body.innerText : "", 8000),
      links,
      fields
    };
    const encoded = btoa(unescape(encodeURIComponent(JSON.stringify(data))));
    try {
      history.replaceState(null, "", location.pathname + location.search + "#__dside_read=" + encoded);
    } catch (_) {
      location.hash = "__dside_read=" + encoded;
    }
  } catch (error) {
    try {
      history.replaceState(null, "", location.pathname + location.search + "#__dside_read_error=1");
    } catch (_) {}
  }
})();
"##;

#[derive(Serialize, Deserialize)]
pub struct BrowserLink {
    pub text: String,
    pub href: String,
}

#[derive(Serialize, Deserialize)]
pub struct BrowserField {
    pub selector: String,
    pub name: String,
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub value: String,
    pub placeholder: String,
    pub checked: bool,
    pub options: Vec<BrowserOption>,
}

#[derive(Serialize, Deserialize)]
pub struct BrowserOption {
    pub value: String,
    pub text: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserPage {
    #[serde(default)]
    pub id: String,
    pub url: String,
    pub title: String,
    pub text: String,
    pub links: Vec<BrowserLink>,
    pub fields: Vec<BrowserField>,
}

#[derive(Serialize, Deserialize)]
pub struct BrowserFillField {
    pub selector: String,
    pub value: String,
}

#[derive(Serialize)]
pub struct BrowserFillResult {
    pub filled: i64,
    pub submitted: bool,
}

fn pick_window(app: &AppHandle, id: Option<&str>) -> Result<WebviewWindow, String> {
    if let Some(id) = id {
        return app
            .get_webview_window(id)
            .ok_or_else(|| format!("browser: no window '{id}'"));
    }
    open_windows(app)
        .into_iter()
        .last()
        .map(|(_, window)| window)
        .ok_or_else(|| "browser: no browser window is open".to_string())
}

fn read_hash(url: &str) -> Option<String> {
    let fragment = url.split_once("#__dside_read=").map(|(_, value)| value)?;
    let bytes = crate::base64_decode(fragment)?;
    String::from_utf8(bytes).ok()
}

fn clear_hash(window: &WebviewWindow) {
    let _ = window.eval(
        "try { history.replaceState(null, '', location.pathname + location.search); } catch (_) {}",
    );
}

/// Reads the page the user has open in our browser window: title, URL, visible text, links and
/// form fields. The page itself never gets IPC access; the script hands the data back through
/// a temporary URL hash, which the host removes again.
#[tauri::command]
pub async fn browser_read(app: AppHandle, id: Option<String>) -> Result<BrowserPage, String> {
    let window = pick_window(&app, id.as_deref())?;
    let label = window.label().to_string();
    window
        .eval(READ_SCRIPT)
        .map_err(|error| format!("browser: {error}"))?;

    for _ in 0..20 {
        sleep(Duration::from_millis(50)).await;
        let url = window
            .url()
            .map(|url| url.to_string())
            .unwrap_or_default();
        if let Some(json) = read_hash(&url) {
            clear_hash(&window);
            let mut page: BrowserPage =
                serde_json::from_str(&json).map_err(|error| format!("browser: {error}"))?;
            page.id = label;
            return Ok(page);
        }
    }

    clear_hash(&window);
    Err("browser: the page did not answer (CSP or an unsupported page)".to_string())
}

fn fill_script(fields: &[BrowserFillField], submit: bool, submit_selector: Option<&str>) -> String {
    let fields_json = serde_json::to_string(fields).unwrap_or_else(|_| "[]".to_string());
    let selector_json = serde_json::to_string(&submit_selector).unwrap_or_else(|_| "null".to_string());
    format!(
        r##"
(() => {{
  try {{
    const fields = {fields_json};
    let filled = 0;
    for (const field of fields) {{
      const el = document.querySelector(field.selector);
      if (!el) continue;
      const type = (el.type || "").toLowerCase();
      if (type === "checkbox" || type === "radio") {{
        el.checked = ["1", "true", "on", "yes", "checked"].includes(String(field.value).toLowerCase());
      }} else if (el.tagName === "SELECT") {{
        el.value = field.value;
        if (el.value !== field.value) {{
          const option = Array.from(el.options).find(
            (entry) => entry.value === field.value || (entry.text || "").trim() === field.value
          );
          if (option) el.value = option.value;
        }}
      }} else {{
        const proto = el.tagName === "TEXTAREA" ? HTMLTextAreaElement.prototype : HTMLInputElement.prototype;
        const setter = Object.getOwnPropertyDescriptor(proto, "value")?.set;
        if (setter) setter.call(el, field.value);
        else el.value = field.value;
      }}
      el.dispatchEvent(new Event("input", {{ bubbles: true }}));
      el.dispatchEvent(new Event("change", {{ bubbles: true }}));
      filled += 1;
    }}
    let submitted = 0;
    if ({}) {{
      const target = {selector_json} ? document.querySelector({selector_json}) : null;
      const form = target && target.tagName === "FORM" ? target : target ? target.closest("form") : document.querySelector("form");
      if (target && target.tagName !== "FORM") {{
        target.click();
      }} else if (form) {{
        if (form.requestSubmit) form.requestSubmit();
        else form.submit();
      }}
      submitted = 1;
    }}
    try {{
      history.replaceState(null, "", location.pathname + location.search + "#__dside_fill=" + filled + ":" + submitted);
    }} catch (_) {{}}
  }} catch (error) {{
    try {{
      history.replaceState(null, "", location.pathname + location.search + "#__dside_fill_error=1");
    }} catch (_) {{}}
  }}
}})();
"##,
        if submit { "true" } else { "false" },
        selector_json = selector_json,
    )
}

/// Fills form controls in the newest browser window. The caller must have been allowed by the
/// user's setting and permission stage; the page itself is only touched by this host script.
#[tauri::command]
pub async fn browser_fill(
    app: AppHandle,
    id: Option<String>,
    fields: Vec<BrowserFillField>,
    submit: Option<bool>,
    submit_selector: Option<String>,
) -> Result<BrowserFillResult, String> {
    if fields.is_empty() && !submit.unwrap_or(false) {
        return Err("browser: nothing to fill".to_string());
    }
    let window = pick_window(&app, id.as_deref())?;
    let should_submit = submit.unwrap_or(false);
    let script = fill_script(&fields, should_submit, submit_selector.as_deref());
    window
        .eval(&script)
        .map_err(|error| format!("browser: {error}"))?;

    let mut filled = fields.len() as i64;
    let mut submitted = should_submit;
    for _ in 0..10 {
        sleep(Duration::from_millis(50)).await;
        let url = window
            .url()
            .map(|url| url.to_string())
            .unwrap_or_default();
        if let Some((count, did_submit)) = url
            .split_once("#__dside_fill=")
            .and_then(|(_, value)| value.split_once(':'))
        {
            filled = count.parse().unwrap_or(filled);
            submitted = did_submit.starts_with('1');
            clear_hash(&window);
            return Ok(BrowserFillResult { filled, submitted });
        }
        if url.contains("#__dside_fill_error=1") {
            clear_hash(&window);
            return Err("browser: the fill script failed on this page".to_string());
        }
    }

    // A submit may already have navigated away, taking the marker with it. The script was
    // accepted by the webview; report the intent rather than pretending it failed.
    if should_submit {
        return Ok(BrowserFillResult { filled, submitted });
    }
    clear_hash(&window);
    Err("browser: the page did not confirm the fill (CSP or an unsupported page)".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_page_hash() {
        let json = r#"{"url":"https://example.test","title":"Example","text":"hi","links":[],"fields":[]}"#;
        let encoded = crate::base64_encode(json.as_bytes());
        let url = format!("https://example.test/#__dside_read={encoded}");
        assert_eq!(read_hash(&url).as_deref(), Some(json));
        assert!(read_hash("https://example.test/").is_none());
    }
}
