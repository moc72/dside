use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;
use tauri::{AppHandle, Manager};

mod browser;
mod db;
mod engine;
mod secret;
mod skills;

// ---------------------------------------------------------------------------
// Editor detection
// ---------------------------------------------------------------------------

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorCandidate {
    id: String,
    label: String,
    family: String,
    exec: String,
    args: Vec<String>,
    source: String,
}

struct KnownEditor {
    id: &'static str,
    label: &'static str,
    family: &'static str,
    bin: &'static str,
    tokens: &'static [&'static str],
}

const KNOWN_EDITORS: &[KnownEditor] = &[
    KnownEditor {
        id: "code-insiders",
        label: "Visual Studio Code Insiders",
        family: "vscode",
        bin: "code-insiders",
        tokens: &["code-insiders", "visual studio code - insiders"],
    },
    KnownEditor {
        id: "code-oss",
        label: "Code - OSS",
        family: "vscode",
        bin: "code-oss",
        tokens: &["code-oss", "code - oss"],
    },
    KnownEditor {
        id: "codium",
        label: "VSCodium",
        family: "vscode",
        bin: "codium",
        tokens: &["codium", "vscodium"],
    },
    KnownEditor {
        id: "code",
        label: "Visual Studio Code",
        family: "vscode",
        bin: "code",
        tokens: &["code", "visual studio code"],
    },
    KnownEditor {
        id: "zed",
        label: "Zed",
        family: "zed",
        bin: "zed",
        tokens: &["zed"],
    },
    KnownEditor {
        id: "cursor",
        label: "Cursor",
        family: "cursor",
        bin: "cursor",
        tokens: &["cursor"],
    },
    KnownEditor {
        id: "windsurf",
        label: "Windsurf",
        family: "windsurf",
        bin: "windsurf",
        tokens: &["windsurf"],
    },
];

fn is_executable_file(path: &Path) -> bool {
    let Ok(meta) = fs::metadata(path) else {
        return false;
    };
    if !meta.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        meta.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

fn split_command_line(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    for ch in input.chars() {
        match quote {
            Some(q) => {
                if ch == q {
                    quote = None;
                } else {
                    current.push(ch);
                }
            }
            None => {
                if ch == '"' || ch == '\'' {
                    quote = Some(ch);
                } else if ch.is_whitespace() {
                    if !current.is_empty() {
                        parts.push(std::mem::take(&mut current));
                    }
                } else {
                    current.push(ch);
                }
            }
        }
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}

fn strip_field_codes(command: &str) -> String {
    split_command_line(command)
        .into_iter()
        .filter(|part| !part.starts_with('%'))
        .collect::<Vec<_>>()
        .join(" ")
}

fn match_known(file_stem: &str, exec_first: &str) -> Option<&'static KnownEditor> {
    let stem = file_stem.to_lowercase();
    let exec = exec_first.to_lowercase();
    let mut best: Option<(&'static KnownEditor, usize)> = None;

    for known in KNOWN_EDITORS {
        for token in known.tokens {
            let token = token.to_lowercase();
            let hit = stem == token
                || stem.starts_with(&format!("{token}."))
                || stem.starts_with(&format!("{token}-"))
                || stem.ends_with(&format!(".{token}"))
                || stem.ends_with(&format!("_{token}"))
                || stem.ends_with(&format!("-{token}"))
                || exec == token;
            if hit && best.map(|(_, len)| token.len() > len).unwrap_or(true) {
                best = Some((known, token.len()));
            }
        }
    }

    best.map(|(known, _)| known)
}

fn desktop_directories() -> Vec<PathBuf> {
    let mut dirs = vec![
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/usr/local/share/applications"),
        PathBuf::from("/var/lib/flatpak/exports/share/applications"),
    ];
    if let Ok(home) = std::env::var("HOME") {
        dirs.push(PathBuf::from(&home).join(".local/share/applications"));
        dirs.push(PathBuf::from(&home).join(".local/share/flatpak/exports/share/applications"));
    }
    dirs
}

fn parse_desktop_entry(content: &str) -> Option<(String, String)> {
    let mut name: Option<String> = None;
    let mut exec: Option<String> = None;
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("Name=") {
            if name.is_none() {
                name = Some(rest.trim().to_string());
            }
        } else if let Some(rest) = line.strip_prefix("Exec=") {
            if exec.is_none() {
                exec = Some(rest.trim().to_string());
            }
        }
    }
    Some((name?, exec?))
}

fn detect_editors_impl() -> Vec<EditorCandidate> {
    let mut found: Vec<EditorCandidate> = Vec::new();

    // 1) Binaries on PATH (native installs).
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            for known in KNOWN_EDITORS {
                let candidate = dir.join(known.bin);
                if is_executable_file(&candidate) {
                    let resolved = fs::canonicalize(&candidate).unwrap_or_else(|_| candidate.clone());
                    found.push(EditorCandidate {
                        id: known.id.to_string(),
                        label: known.label.to_string(),
                        family: known.family.to_string(),
                        exec: resolved.to_string_lossy().to_string(),
                        args: Vec::new(),
                        source: "path".to_string(),
                    });
                }
            }
        }
    }

    // 2) Desktop entries (covers Flatpak/Snap/AppImage and non-PATH installs).
    for dir in desktop_directories() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if !file_name.ends_with(".desktop") {
                continue;
            }
            let stem = file_name.trim_end_matches(".desktop").to_string();
            if stem.ends_with("-url-handler") {
                continue;
            }
            let Ok(content) = fs::read_to_string(entry.path()) else {
                continue;
            };
            let Some((_name, exec)) = parse_desktop_entry(&content) else {
                continue;
            };
            let parts = split_command_line(&exec);
            if parts
                .iter()
                .any(|part| part == "--open-url" || part == "--unity-launch")
            {
                continue;
            }
            let Some(first) = parts.first() else { continue };
            if first.ends_with(".desktop") {
                continue;
            }
            let Some(known) = match_known(&stem, first) else {
                continue;
            };
            let source = if exec.contains("flatpak") {
                "flatpak"
            } else {
                "desktop"
            };
            found.push(EditorCandidate {
                id: known.id.to_string(),
                label: known.label.to_string(),
                family: known.family.to_string(),
                exec: strip_field_codes(&exec),
                args: Vec::new(),
                source: source.to_string(),
            });
        }
    }

    // 3) Deduplicate by resolved program, then disambiguate repeated labels.
    let mut best: Vec<EditorCandidate> = Vec::new();
    for candidate in found {
        let key = dedupe_key(&candidate);
        match best
            .iter()
            .position(|existing| dedupe_key(existing) == key)
        {
            Some(index) => {
                if rank(&candidate) < rank(&best[index]) {
                    best[index] = candidate;
                }
            }
            None => best.push(candidate),
        }
    }
    let mut unique = best;

    let mut seen_labels: Vec<String> = Vec::new();
    for candidate in unique.iter_mut() {
        if seen_labels.iter().any(|label| label == &candidate.label) {
            candidate.label = format!("{} ({})", candidate.label, source_label(&candidate.source));
        }
        seen_labels.push(candidate.label.clone());
    }

    unique.sort_by(|a, b| {
        rank(a).cmp(&rank(b)).then_with(|| a.label.cmp(&b.label))
    });
    unique
}

fn source_label(source: &str) -> &'static str {
    match source {
        "path" => "native",
        "flatpak" => "Flatpak",
        "desktop" => "Desktop",
        _ => "other",
    }
}

fn rank(candidate: &EditorCandidate) -> u8 {
    match candidate.source.as_str() {
        "path" => 0,
        "flatpak" => 1,
        "desktop" => 2,
        _ => 3,
    }
}

fn resolve_program(program: &str) -> Option<PathBuf> {
    if program.contains('/') {
        return fs::canonicalize(program).ok();
    }
    let path_var = std::env::var("PATH").ok()?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(program);
        if is_executable_file(&candidate) {
            return Some(fs::canonicalize(&candidate).unwrap_or(candidate));
        }
    }
    None
}

fn dedupe_key(candidate: &EditorCandidate) -> String {
    let program = split_command_line(&candidate.exec)
        .into_iter()
        .next()
        .unwrap_or_default();
    let base = Path::new(&program)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| program.clone());

    // Flatpak/Snap share one launcher binary, so the app identity lives in the
    // arguments — keep the full command line as the key.
    if base == "flatpak" || base == "snap" {
        return candidate.exec.clone();
    }

    resolve_program(&program)
        .map(|resolved| resolved.to_string_lossy().to_string())
        .unwrap_or(program)
}

#[tauri::command]
fn detect_editors() -> Vec<EditorCandidate> {
    let found = detect_editors_impl();
    // One line per call: "no editor found" in the UI is otherwise impossible to tell apart
    // from "the scan found nothing in this environment".
    eprintln!(
        "editors: {} found [{}]",
        found.len(),
        found
            .iter()
            .map(|candidate| candidate.id.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    found
}

#[tauri::command]
fn open_workspace_in_editor(editor_id: String, workspace: String) -> Result<(), String> {
    let candidates = detect_editors_impl();
    let chosen = candidates
        .into_iter()
        .find(|candidate| candidate.id == editor_id)
        .ok_or_else(|| format!("editor not found: {editor_id}"))?;

    let mut parts = split_command_line(&chosen.exec);
    if parts.is_empty() {
        return Err("empty editor command".to_string());
    }
    let program = parts.remove(0);
    let mut args = parts;
    args.extend(chosen.args);
    if !workspace.is_empty() {
        args.push(workspace);
    }

    Command::new(program)
        .args(args)
        .spawn()
        .map_err(|error| error.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Workspace filesystem
// ---------------------------------------------------------------------------

const IGNORED_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    "target",
    "dist",
    ".cache",
    "__pycache__",
    ".venv",
    "venv",
    ".svn",
    ".hg",
    ".pnpm-store",
];

const MAX_CHILDREN: usize = 400;
const MAX_FILE_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeDto {
    name: String,
    path: String,
    is_folder: bool,
    children: Vec<NodeDto>,
    expanded: bool,
    language: Option<String>,
    /// Whether `children` were actually read. Folders beyond the depth limit are not
    /// loaded yet, and the frontend fetches them when the user expands them.
    loaded: bool,
}

fn read_dir_nodes(dir: &Path, depth: u32) -> Vec<NodeDto> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut collected: Vec<(String, PathBuf, bool)> = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if IGNORED_DIRS.contains(&name.as_str()) {
            continue;
        }
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_symlink() {
            continue;
        }
        collected.push((name, entry.path(), file_type.is_dir()));
    }

    collected.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

    let mut folders: Vec<NodeDto> = Vec::new();
    let mut files: Vec<NodeDto> = Vec::new();

    for (name, path, is_folder) in collected.into_iter().take(MAX_CHILDREN) {
        let descend = is_folder && depth > 0;
        let children = if descend {
            read_dir_nodes(&path, depth - 1)
        } else {
            Vec::new()
        };
        let node = NodeDto {
            name,
            path: path.to_string_lossy().to_string(),
            is_folder,
            children,
            expanded: false,
            language: None,
            loaded: descend,
        };
        if is_folder {
            folders.push(node);
        } else {
            files.push(node);
        }
    }

    folders.extend(files);
    folders
}

#[tauri::command]
fn read_workspace_tree(root: String, depth: Option<u32>) -> Result<Vec<NodeDto>, String> {
    let path = PathBuf::from(&root);
    if !path.is_dir() {
        return Err(format!("not a directory: {root}"));
    }
    Ok(read_dir_nodes(&path, depth.unwrap_or(1).min(6)))
}

/// One directory level. The file tree calls this when a folder is expanded, so opening a
/// small subfolder never means walking the whole tree first.
#[tauri::command]
fn read_dir_children(path: String) -> Result<Vec<NodeDto>, String> {
    let target = PathBuf::from(&path);
    if !target.is_dir() {
        return Err(format!("not a directory: {path}"));
    }
    Ok(read_dir_nodes(&target, 0))
}

#[tauri::command]
fn read_text_file(path: String) -> Result<String, String> {
    let target = PathBuf::from(&path);
    let meta = fs::metadata(&target).map_err(|error| error.to_string())?;
    if !meta.is_file() {
        return Err("not a file".to_string());
    }
    if meta.len() > MAX_FILE_BYTES {
        return Err("file too large".to_string());
    }
    fs::read_to_string(&target).map_err(|error| error.to_string())
}

/// Reads a binary file as base64 so the webview can preview it (PDF). Capped, so a stray
/// click on a huge file cannot eat the webview's memory.
#[tauri::command]
fn read_file_base64(path: String) -> Result<String, String> {
    const MAX_BYTES: u64 = 40 * 1024 * 1024;

    let target = PathBuf::from(&path);
    let meta = std::fs::metadata(&target).map_err(|error| error.to_string())?;
    if !meta.is_file() {
        return Err("not a file".to_string());
    }
    if meta.len() > MAX_BYTES {
        return Err("file too large for preview".to_string());
    }

    let bytes = std::fs::read(&target).map_err(|error| error.to_string())?;
    Ok(base64_encode(&bytes))
}

/// Hand-rolled to avoid another dependency; sixteen lines and easy to test.
pub(crate) fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);

    for chunk in bytes.chunks(3) {
        let first = chunk[0] as u32;
        let second = *chunk.get(1).unwrap_or(&0) as u32;
        let third = *chunk.get(2).unwrap_or(&0) as u32;
        let combined = (first << 16) | (second << 8) | third;

        out.push(TABLE[(combined >> 18) as usize & 63] as char);
        out.push(TABLE[(combined >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            TABLE[(combined >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[combined as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

#[tauri::command]
fn default_workspace() -> String {
    std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
}

/// The name pool the partner may browse. Embedded at compile time, so it works the same in a
/// packaged build as in the repository — no path to get wrong.
const NAME_POOL: &str = include_str!("../../src/assets/namen.json");

#[derive(serde::Deserialize)]
struct NameIdea {
    name: String,
    origin: String,
    meaning: String,
}

fn search_names(needle: Option<&str>, origin: Option<&str>, limit: usize) -> Vec<String> {
    let pool: Vec<NameIdea> = match serde_json::from_str(NAME_POOL) {
        Ok(pool) => pool,
        Err(_) => return Vec::new(),
    };
    let needle = needle.map(str::to_lowercase).filter(|text| !text.is_empty());
    let origin = origin.map(str::to_lowercase).filter(|text| !text.is_empty());

    pool.into_iter()
        .filter(|entry| {
            needle.as_ref().is_none_or(|text| entry.meaning.to_lowercase().contains(text))
        })
        .filter(|entry| {
            origin.as_ref().is_none_or(|text| entry.origin.to_lowercase().contains(text))
        })
        .take(limit)
        .map(|entry| format!("{} ({}) — {}", entry.name, entry.origin, entry.meaning))
        .collect()
}

/// Names to choose from, by what they mean. Not a menu: a place to look when nothing comes
/// to mind. Kept out of the system prompt on purpose — 947 entries would ride along in every
/// single request.
/// Writes an attachment next to the database and hands back its path. The conversation keeps
/// the path, not the bytes: a chat log should not grow by megabytes per picture.
#[tauri::command]
fn attachment_save(app: AppHandle, name: String, mime: String, data_base64: String) -> Result<String, String> {
    let directory = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("attachments");
    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;

    let bytes = base64_decode(&data_base64).ok_or_else(|| "attachment: unreadable data".to_string())?;
    // The content type decides the extension: it is what the file actually is, while a name
    // can say anything.
    let extension = match mime.as_str() {
        "image/png" => "png".to_string(),
        "image/jpeg" => "jpg".to_string(),
        "image/webp" => "webp".to_string(),
        "image/gif" => "gif".to_string(),
        _ => name
            .rsplit('.')
            .next()
            .filter(|value| value.len() <= 5 && value.chars().all(|c| c.is_ascii_alphanumeric()))
            .unwrap_or("bin")
            .to_lowercase(),
    };
    let stem: String = name
        .trim_end_matches(&format!(".{extension}"))
        .chars()
        .map(|character| if character.is_ascii_alphanumeric() { character } else { '-' })
        .collect();
    let stem = stem.trim_matches('-').chars().take(40).collect::<String>();
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let file = directory.join(format!("{stamp}-{stem}.{extension}"));
    std::fs::write(&file, bytes).map_err(|error| error.to_string())?;
    Ok(file.to_string_lossy().to_string())
}

fn base64_decode(value: &str) -> Option<Vec<u8>> {
    fn digit(character: u8) -> Option<u8> {
        match character {
            b'A'..=b'Z' => Some(character - b'A'),
            b'a'..=b'z' => Some(character - b'a' + 26),
            b'0'..=b'9' => Some(character - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }
    let cleaned: Vec<u8> = value
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect();
    let mut out = Vec::with_capacity(cleaned.len() / 4 * 3);
    for chunk in cleaned.chunks(4) {
        let mut buffer = 0u32;
        let mut count = 0;
        for (index, byte) in chunk.iter().enumerate() {
            if *byte == b'=' {
                break;
            }
            buffer |= (digit(*byte)? as u32) << (18 - 6 * index);
            count += 1;
        }
        if count >= 2 { out.push((buffer >> 16) as u8); }
        if count >= 3 { out.push((buffer >> 8) as u8); }
        if count >= 4 { out.push(buffer as u8); }
    }
    Some(out)
}

#[tauri::command]
fn name_ideas(meaning: Option<String>, origin: Option<String>, limit: Option<usize>) -> Vec<String> {
    search_names(meaning.as_deref(), origin.as_deref(), limit.unwrap_or(40).min(200))
}

/// What the About page shows: the version and the packages the app is actually built from.
///
/// Both manifests are embedded at compile time, so the list cannot go stale the way a
/// hand-maintained one would — and there is no build step to forget.
#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct AboutPackage {
    name: String,
    version: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct AboutInfo {
    version: String,
    frontend: Vec<AboutPackage>,
    backend: Vec<AboutPackage>,
}

#[tauri::command]
fn about_info() -> AboutInfo {
    AboutInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        frontend: frontend_packages(include_str!("../../package.json")),
        backend: backend_packages(include_str!("../Cargo.toml")),
    }
}

fn tidy_version(version: &str) -> String {
    version.trim().trim_start_matches(['^', '~', '=']).to_string()
}

fn frontend_packages(manifest: &str) -> Vec<AboutPackage> {
    let parsed: serde_json::Value = match serde_json::from_str(manifest) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };

    let mut packages: Vec<AboutPackage> = parsed
        .get("dependencies")
        .and_then(|value| value.as_object())
        .map(|dependencies| {
            dependencies
                .iter()
                .map(|(name, version)| AboutPackage {
                    name: name.clone(),
                    version: tidy_version(version.as_str().unwrap_or("")),
                })
                .collect()
        })
        .unwrap_or_default();
    packages.sort_by(|a, b| a.name.cmp(&b.name));
    packages
}

/// Reads the `[dependencies]` block of the crate manifest. Enough TOML for our own file, and
/// it names the crates the way a reader expects them rather than by their full spec.
fn backend_packages(manifest: &str) -> Vec<AboutPackage> {
    let mut packages = Vec::new();
    let mut section = String::new();

    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            section = line.trim_matches(['[', ']']).to_string();
            continue;
        }
        if section != "dependencies" || line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((name, value)) = line.split_once('=') else {
            continue;
        };

        // Either `crate = "1"` or `crate = { version = "1", features = [...] }`.
        let value = value.trim();
        let version = if value.starts_with('"') {
            value.split('"').nth(1).unwrap_or("").to_string()
        } else {
            value
                .split_once("version")
                .map(|(_, tail)| tail)
                .and_then(|tail| tail.split('"').nth(1))
                .unwrap_or("")
                .to_string()
        };

        packages.push(AboutPackage {
            name: name.trim().trim_matches('"').to_string(),
            version: if version.is_empty() {
                "—".to_string()
            } else {
                tidy_version(&version)
            },
        });
    }

    packages.sort_by(|a, b| a.name.cmp(&b.name));
    packages
}

/// A channel for the frontend to write into the same log. A desktop app without a console
/// needs one, and "the UI did nothing" is otherwise indistinguishable from "the UI never ran".
#[tauri::command]
fn log_message(text: String) {
    eprintln!("ui: {text}");
}

/// Shows the splash only once its artwork is on screen — the window is created hidden for
/// exactly this reason, so nobody sees a black rectangle while the webview boots.
#[tauri::command]
fn splash_visible(app: AppHandle) {
    if let Some(splash) = app.get_webview_window("splash") {
        let _ = splash.show();
        let _ = splash.set_focus();
        eprintln!("startup: splash shown");
    }
}

/// Called by the splash window once it has painted. Idempotent, so a late call is harmless.
#[tauri::command]
fn splash_ready(app: AppHandle) {
    show_main_window(&app);
}

fn show_main_window(app: &AppHandle) {
    if let Some(splash) = app.get_webview_window("splash") {
        let _ = splash.close();
        eprintln!("startup: splash closed");
    }
    if let Some(main) = app.get_webview_window("main") {
        if !main.is_visible().unwrap_or(false) {
            let _ = main.show();
            let _ = main.set_focus();
            eprintln!("startup: main window shown");
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // Size, position and maximised state of the main window survive a restart. Visibility
        // is deliberately left out: the splash decides when the main window appears, and a
        // restored "visible" flag would show it over the splash.
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(
                    tauri_plugin_window_state::StateFlags::SIZE
                        | tauri_plugin_window_state::StateFlags::POSITION
                        | tauri_plugin_window_state::StateFlags::MAXIMIZED,
                )
                .with_denylist(&["splash"])
                .build(),
        )
        .manage(engine::EngineState::default())
        .setup(|app| {
            let directory = app.path().app_data_dir()?;
            let database = db::Db::open(&directory.join("ds-ide.sqlite3"))?;
            app.manage(database);
            app.manage(secret::SecretStore::new(directory.join("secrets.json")));

            // Interval scheduler: check for due runs every 30 seconds. It only lives as
            // long as the app does — there is no OS daemon.
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                    engine::run_due_schedules(&handle).await;
                }
            });

            // Splash: the main window stays hidden until the splash says it has painted.
            // The fallback makes sure a broken frontend cannot leave the user windowless.
            let fallback_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(12)).await;
                if let Some(main) = fallback_handle.get_webview_window("main") {
                    if !main.is_visible().unwrap_or(false) {
                        eprintln!("startup: splash fallback fired");
                        show_main_window(&fallback_handle);
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            detect_editors,
            open_workspace_in_editor,
            read_workspace_tree,
            read_dir_children,
            read_text_file,
            read_file_base64,
            default_workspace,
            engine::engine_set_config,
            engine::engine_configured,
            engine::engine_send,
            engine::session_compact,
            engine::engine_models,
            engine::provider_balance,
            browser::open_browser_window,
            browser::browser_windows,
            browser::browser_action,
            browser::browser_close,
            splash_visible,
            log_message,
            splash_ready,
            engine::engine_answer,
            engine::engine_abort,
            engine::engine_reset,
            db::sessions_list,
            db::sessions_archived,
            db::session_archive,
            db::session_create,
            db::session_history,
            db::session_delete,
            db::session_update,
            db::session_todos,
            db::settings_all,
            db::settings_set,
            db::partner_name,
            db::partner_name_set,
            about_info,
            db::default_display_name,
            db::memories_list,
            db::memories_retired,
            db::memory_add,
            db::memory_update,
            db::memory_retire,
            db::memory_restore,
            name_ideas,
            attachment_save,
            db::memory_report,
            db::memory_prune,
            db::search,
            db::schedules_list,
            db::schedule_create,
            db::schedule_delete,
            db::schedule_toggle,
            skills::skills_list,
            skills::skills_root_path,
            skills::skill_read,
            skills::skill_write,
            secret::secret_status,
            secret::secret_set,
            secret::secret_clear,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The About page reads the manifests, so a broken parser would show an empty list and
    /// nobody would notice until someone opened it.
    #[test]
    fn about_reads_both_manifests() {
        let frontend = frontend_packages(include_str!("../../package.json"));
        assert!(
            frontend.iter().any(|entry| entry.name == "vue" && !entry.version.is_empty()),
            "frontend packages: {frontend:?}"
        );
        assert!(
            frontend.iter().all(|entry| !entry.version.starts_with('^')),
            "versions are shown without the range prefix: {frontend:?}"
        );

        let backend = backend_packages(include_str!("../Cargo.toml"));
        assert!(
            backend.iter().any(|entry| entry.name == "tauri" && entry.version == "2"),
            "tauri version: {backend:?}"
        );
        assert!(
            backend.iter().any(|entry| entry.name == "rusqlite" && !entry.version.is_empty()),
            "named dependency with a version table: {backend:?}"
        );
        assert!(
            !backend.iter().any(|entry| entry.name.contains("dependencies")),
            "sections are not packages: {backend:?}"
        );
    }

    /// The pool is embedded and searched by meaning; if the file or the field names ever
    /// change, this fails instead of the tool quietly returning nothing.
    #[test]
    fn the_name_pool_can_be_searched() {
        let all = search_names(None, None, 5000);
        assert!(all.len() > 900, "pool size: {}", all.len());
        assert!(all.iter().all(|line| line.contains(" — ")), "each line carries a meaning");

        let light = search_names(Some("licht"), None, 40);
        assert!(!light.is_empty());
        assert!(light.iter().all(|line| line.to_lowercase().contains("licht")));

        let nordic = search_names(None, Some("nordisch"), 200);
        assert!(!nordic.is_empty(), "origin filter works");

        assert!(search_names(Some("zzzz"), None, 10).is_empty());
        assert_eq!(search_names(None, None, 3).len(), 3, "limit is respected");
    }

    #[test]
    fn detect_editors_smoke() {
        let found = detect_editors_impl();
        println!("detected {} editor installation(s):", found.len());
        for candidate in &found {
            println!(
                "  {:<14} {:<28} {:<8} {}",
                candidate.id, candidate.label, candidate.source, candidate.exec
            );
        }
    }

    /// RFC 4648 test vectors — a hand-rolled encoder is exactly the place where an
    /// off-by-one hides until a PDF fails to load.
    #[test]
    fn base64_matches_known_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    /// Compares against the system encoder on a real binary file. Opt-in, because it needs
    /// a path from the user's disk: `DS_IDE_TEST_IMAGE=/path/to.png cargo test -- --ignored`.
    #[test]
    #[ignore = "needs DS_IDE_TEST_IMAGE"]
    fn base64_matches_the_system_on_a_real_file() {
        let Ok(path) = std::env::var("DS_IDE_TEST_IMAGE") else {
            return;
        };
        let bytes = std::fs::read(&path).expect("read test image");
        let ours = base64_encode(&bytes);

        let reference = std::process::Command::new("base64")
            .arg("-w")
            .arg("0")
            .arg(&path)
            .output()
            .expect("run system base64");
        let expected = String::from_utf8(reference.stdout).expect("system base64 is utf-8");

        assert_eq!(ours, expected.trim(), "encoder differs on {path}");
        println!("{} bytes encoded identically", bytes.len());
    }
}
