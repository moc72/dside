use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use regex::Regex;
use serde_json::{json, Value};
use tokio::process::Command;

pub struct ToolContext {
    pub workspace: PathBuf,
    pub full_authorization: bool,
    pub skills_root: PathBuf,
}

pub struct ToolOutcome {
    pub content: String,
    pub is_error: bool,
}

impl ToolOutcome {
    pub fn ok(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            is_error: false,
        }
    }

    pub fn err(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            is_error: true,
        }
    }
}

const MAX_READ_LINES: usize = 2000;
const MAX_OUTPUT_CHARS: usize = 30_000;
const MAX_WALK_DEPTH: usize = 8;
const MAX_WALK_FILE_BYTES: u64 = 1_500_000;
const IGNORED_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    "target",
    "dist",
    ".cache",
    "__pycache__",
    ".venv",
    "venv",
];

/// Tool definitions in OpenAI function-calling shape.
/// Names mirror MiniMax Code's `Local*ToolDef` set (read/write/edit/bash/grep/glob/todowrite).
pub fn definitions() -> Vec<Value> {
    vec![
        json!({
            "type": "function",
            "function": {
                "name": "read",
                "description": "Read a text file. Returns lines prefixed with their 1-based line number. Use offset/limit for large files.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Workspace-relative or absolute file path." },
                        "offset": { "type": "number", "description": "First line to read (1-indexed)." },
                        "limit": { "type": "number", "description": "Maximum number of lines to read." }
                    },
                    "required": ["path"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "write",
                "description": "Write a file, creating parent directories and overwriting existing content.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Workspace-relative or absolute file path." },
                        "content": { "type": "string", "description": "Full file content to write." }
                    },
                    "required": ["path", "content"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "edit",
                "description": "Replace an exact string in a file. old_string must be unique unless replace_all is true.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "File to edit." },
                        "old_string": { "type": "string", "description": "Exact text to replace." },
                        "new_string": { "type": "string", "description": "Replacement text." },
                        "replace_all": { "type": "boolean", "description": "Replace every occurrence." }
                    },
                    "required": ["path", "old_string", "new_string"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "bash",
                "description": "Run a shell command with sh -c in the workspace directory. Returns exit code, stdout and stderr.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": { "type": "string", "description": "Shell command to run." },
                        "timeout_ms": { "type": "number", "description": "Timeout in milliseconds (default 120000)." }
                    },
                    "required": ["command"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "glob",
                "description": "Find files by glob pattern, e.g. \"src/**/*.ts\".",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "pattern": { "type": "string", "description": "Glob pattern." },
                        "path": { "type": "string", "description": "Directory to search in (default: workspace)." }
                    },
                    "required": ["pattern"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "grep",
                "description": "Search file contents with a regular expression. Returns path:line: text.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "pattern": { "type": "string", "description": "Regular expression." },
                        "path": { "type": "string", "description": "Directory to search (default: workspace)." },
                        "include": { "type": "string", "description": "Only files whose name ends with this suffix, e.g. \".ts\"." }
                    },
                    "required": ["pattern"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "todowrite",
                "description": "Store the current task list so long tasks keep a visible plan.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "todos": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "content": { "type": "string" },
                                    "status": { "type": "string" },
                                    "priority": { "type": "string" }
                                }
                            }
                        }
                    },
                    "required": ["todos"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "memory",
                "description": "Long-term memory, kept across sessions. Use 'add' to remember a durable fact, preference, event or project detail about the user or about yourself; 'list' to review what is stored; 'forget' to delete an entry by id; 'curate' to get a hygiene report (exact duplicates and long-unused entries) and then merge or forget deliberately.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "action": { "type": "string", "enum": ["add", "list", "forget", "curate"] },
                        "kind": { "type": "string", "enum": ["fact", "preference", "event", "project", "news"] },
                        "scope": { "type": "string", "enum": ["user", "agent", "session"] },
                        "text": { "type": "string", "description": "The memory, one sentence." },
                        "id": { "type": "number", "description": "Entry id, required for 'forget'." }
                    },
                    "required": ["action"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "skill",
                "description": "Available skills: reusable instructions for a class of task (e.g. landing pages, PDFs). Call with action 'list' to see what is installed, or with 'name' to load a skill and follow it.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "action": { "type": "string", "enum": ["list", "load"] },
                        "name": { "type": "string", "description": "Skill name, required for 'load'." }
                    },
                    "required": ["action"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "web_fetch",
                "description": "Fetch a URL over HTTP(S) and return its readable text (HTML tags stripped). Use it to read documentation or a page the user linked.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "url": { "type": "string", "description": "Absolute http(s) URL." }
                    },
                    "required": ["url"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "code_review",
                "description": "Collect the current git diff (optionally staged, optionally a path) and return it for review: stat plus patch.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "staged": { "type": "boolean", "description": "Review staged changes instead of the working tree." },
                        "path": { "type": "string", "description": "Limit the diff to this path." }
                    }
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "ask_user",
                "description": "Ask the user a question and wait for the answer. Use it when a decision materially changes the outcome and you cannot infer it. Provide short options when the answer is a choice.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "question": { "type": "string", "description": "The question, one sentence." },
                        "options": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Optional suggested answers."
                        }
                    },
                    "required": ["question"]
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "task",
                "description": "Run a self-contained sub-task in a fresh context and get only its final answer back. Use it for work that would otherwise flood your own context (broad searches, long reading, isolated research). The sub-agent cannot see this conversation, so its prompt must be complete on its own. Use sparingly: each run costs its own tokens.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "prompt": { "type": "string", "description": "Complete, self-contained instruction for the sub-agent." },
                        "description": { "type": "string", "description": "Short label shown to the user, three to five words." }
                    },
                    "required": ["prompt"]
                }
            }
        }),
    ]
}

fn resolve(workspace: &Path, input: &str) -> PathBuf {
    let path = Path::new(input);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace.join(path)
    }
}

fn truncate(text: String, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text;
    }
    let mut out: String = text.chars().take(limit).collect();
    out.push_str("\n… [output truncated]");
    out
}

fn arg<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(Value::as_str)
}

pub async fn execute(
    name: &str,
    args: &Value,
    ctx: &ToolContext,
    todos: &mut Vec<Value>,
) -> ToolOutcome {
    // The permission gate: tools that touch disk or run processes need
    // "Full Authorization". The UI toggle is the switch; deny is explicit so
    // the model can tell the user what to enable.
    if !ctx.full_authorization && matches!(name, "write" | "edit" | "bash") {
        return ToolOutcome::err(format!(
            "permission denied: '{name}' requires Full Authorization — ask the user to enable it"
        ));
    }

    match name {
        "read" => read(args, ctx),
        "write" => write(args, ctx),
        "edit" => edit(args, ctx),
        "glob" => glob_tool(args, ctx),
        "grep" => grep(args, ctx),
        "todowrite" => todowrite(args, todos),
        "skill" => skill_tool(args, ctx),
        "web_fetch" => web_fetch(args).await,
        "code_review" => code_review(args, ctx).await,
        "bash" => bash(args, ctx).await,
        other => ToolOutcome::err(format!("unknown tool: {other}")),
    }
}

fn read(args: &Value, ctx: &ToolContext) -> ToolOutcome {
    let Some(raw_path) = arg(args, "path") else {
        return ToolOutcome::err("read: missing 'path'");
    };
    let target = resolve(&ctx.workspace, raw_path);
    let content = match std::fs::read_to_string(&target) {
        Ok(content) => content,
        Err(error) => return ToolOutcome::err(format!("read: {error}")),
    };

    let offset = args.get("offset").and_then(Value::as_u64).unwrap_or(1).max(1) as usize;
    let limit = args
        .get("limit")
        .and_then(Value::as_u64)
        .map(|value| value as usize)
        .unwrap_or(MAX_READ_LINES);

    let total_lines = content.lines().count();
    let mut out = String::new();
    let mut written = 0usize;
    for (index, line) in content.lines().enumerate() {
        let number = index + 1;
        if number < offset {
            continue;
        }
        if written >= limit {
            break;
        }
        out.push_str(&format!("{number:>5}\t{line}\n"));
        written += 1;
    }

    if total_lines == 0 {
        return ToolOutcome::ok(format!("(empty file: {})", target.display()));
    }
    if out.is_empty() {
        return ToolOutcome::ok(format!(
            "(no lines in range; file has {total_lines} lines)"
        ));
    }
    if offset + written <= total_lines {
        out.push_str(&format!("… [{} more lines]\n", total_lines - (offset + written - 1)));
    }
    ToolOutcome::ok(truncate(out, MAX_OUTPUT_CHARS))
}

fn write(args: &Value, ctx: &ToolContext) -> ToolOutcome {
    let Some(raw_path) = arg(args, "path") else {
        return ToolOutcome::err("write: missing 'path'");
    };
    let content = arg(args, "content").unwrap_or("");
    let target = resolve(&ctx.workspace, raw_path);

    if let Some(parent) = target.parent() {
        if let Err(error) = std::fs::create_dir_all(parent) {
            return ToolOutcome::err(format!("write: {error}"));
        }
    }
    match std::fs::write(&target, content) {
        Ok(()) => ToolOutcome::ok(format!(
            "wrote {} byte(s) to {}",
            content.len(),
            target.display()
        )),
        Err(error) => ToolOutcome::err(format!("write: {error}")),
    }
}

fn edit(args: &Value, ctx: &ToolContext) -> ToolOutcome {
    let Some(raw_path) = arg(args, "path") else {
        return ToolOutcome::err("edit: missing 'path'");
    };
    let Some(old_string) = arg(args, "old_string") else {
        return ToolOutcome::err("edit: missing 'old_string'");
    };
    let new_string = arg(args, "new_string").unwrap_or("");
    let replace_all = args
        .get("replace_all")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let target = resolve(&ctx.workspace, raw_path);

    let content = match std::fs::read_to_string(&target) {
        Ok(content) => content,
        Err(error) => return ToolOutcome::err(format!("edit: {error}")),
    };

    let occurrences = content.matches(old_string).count();
    if occurrences == 0 {
        return ToolOutcome::err("edit: old_string not found");
    }
    if occurrences > 1 && !replace_all {
        return ToolOutcome::err(format!(
            "edit: old_string is not unique ({occurrences} matches) — add more context or set replace_all"
        ));
    }

    let updated = if replace_all {
        content.replace(old_string, new_string)
    } else {
        content.replacen(old_string, new_string, 1)
    };

    match std::fs::write(&target, updated) {
        Ok(()) => ToolOutcome::ok(format!(
            "edited {} ({occurrences} replacement(s))",
            target.display()
        )),
        Err(error) => ToolOutcome::err(format!("edit: {error}")),
    }
}

fn glob_tool(args: &Value, ctx: &ToolContext) -> ToolOutcome {
    let Some(pattern) = arg(args, "pattern") else {
        return ToolOutcome::err("glob: missing 'pattern'");
    };
    let base = arg(args, "path")
        .map(|value| resolve(&ctx.workspace, value))
        .unwrap_or_else(|| ctx.workspace.clone());
    let joined = base.join(pattern).to_string_lossy().to_string();

    let entries = match glob::glob(&joined) {
        Ok(entries) => entries,
        Err(error) => return ToolOutcome::err(format!("glob: {error}")),
    };

    let mut results: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        results.push(entry.display().to_string());
        if results.len() >= 300 {
            break;
        }
    }

    if results.is_empty() {
        return ToolOutcome::ok("(no matches)");
    }
    ToolOutcome::ok(truncate(results.join("\n"), MAX_OUTPUT_CHARS))
}

fn grep(args: &Value, ctx: &ToolContext) -> ToolOutcome {
    let Some(pattern) = arg(args, "pattern") else {
        return ToolOutcome::err("grep: missing 'pattern'");
    };
    let include = arg(args, "include");
    let root = arg(args, "path")
        .map(|value| resolve(&ctx.workspace, value))
        .unwrap_or_else(|| ctx.workspace.clone());

    let regex = match Regex::new(pattern) {
        Ok(regex) => regex,
        Err(error) => return ToolOutcome::err(format!("grep: {error}")),
    };

    let mut hits: Vec<String> = Vec::new();
    walk_grep(&root, include, &regex, &mut hits, 0);
    if hits.is_empty() {
        return ToolOutcome::ok("(no matches)");
    }
    ToolOutcome::ok(truncate(hits.join("\n"), MAX_OUTPUT_CHARS))
}

fn walk_grep(dir: &Path, include: Option<&str>, regex: &Regex, hits: &mut Vec<String>, depth: usize) {
    if depth > MAX_WALK_DEPTH || hits.len() >= 300 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if hits.len() >= 300 {
            return;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if IGNORED_DIRS.contains(&name.as_str()) {
            continue;
        }
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let path = entry.path();
        if file_type.is_dir() {
            walk_grep(&path, include, regex, hits, depth + 1);
            continue;
        }
        if let Some(suffix) = include {
            if !name.ends_with(suffix) {
                continue;
            }
        }
        let Ok(meta) = entry.metadata() else { continue };
        if meta.len() > MAX_WALK_FILE_BYTES {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        for (index, line) in content.lines().enumerate() {
            if regex.is_match(line) {
                hits.push(format!("{}:{}: {}", path.display(), index + 1, line.trim()));
                if hits.len() >= 300 {
                    return;
                }
            }
        }
    }
}

fn todowrite(args: &Value, todos: &mut Vec<Value>) -> ToolOutcome {
    let list = args
        .get("todos")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let count = list.len();
    *todos = list;
    ToolOutcome::ok(format!("{count} todo(s) stored"))
}

fn skill_tool(args: &Value, ctx: &ToolContext) -> ToolOutcome {
    let action = args.get("action").and_then(Value::as_str).unwrap_or("list");
    if action != "load" {
        let skills = crate::skills::list();
        if skills.is_empty() {
            return ToolOutcome::ok(format!(
                "(no skills installed — drop SKILL.md files into {})",
                ctx.skills_root.display()
            ));
        }
        let body = skills
            .iter()
            .map(|skill| format!("- {} — {}", skill.name, skill.description))
            .collect::<Vec<_>>()
            .join("\n");
        return ToolOutcome::ok(body);
    }

    let Some(name) = arg(args, "name") else {
        return ToolOutcome::err("skill: 'name' is required for load");
    };
    match crate::skills::find(name) {
        Some(path) => match std::fs::read_to_string(&path) {
            Ok(content) => ToolOutcome::ok(truncate(content, MAX_OUTPUT_CHARS)),
            Err(error) => ToolOutcome::err(format!("skill: {error}")),
        },
        None => ToolOutcome::err(format!("skill: '{name}' not found")),
    }
}

async fn web_fetch(args: &Value) -> ToolOutcome {
    let Some(url) = arg(args, "url") else {
        return ToolOutcome::err("web_fetch: missing 'url'");
    };
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return ToolOutcome::err("web_fetch: only http(s) URLs are supported");
    }

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("DS.IDE/0.1")
        .build()
    {
        Ok(client) => client,
        Err(error) => return ToolOutcome::err(format!("web_fetch: {error}")),
    };

    let response = match client.get(url).send().await {
        Ok(response) => response,
        Err(error) => return ToolOutcome::err(format!("web_fetch: {error}")),
    };
    let status = response.status();
    let body = match response.text().await {
        Ok(body) => body,
        Err(error) => return ToolOutcome::err(format!("web_fetch: {error}")),
    };

    let text = html_to_text(&body);
    if text.trim().is_empty() {
        return ToolOutcome::ok(format!("HTTP {status} — no readable text"));
    }
    ToolOutcome::ok(truncate(format!("HTTP {status}\n\n{text}"), MAX_OUTPUT_CHARS))
}

/// Deliberately crude: drop script/style blocks, strip tags, decode the common entities.
/// A real DOM parser would be nicer, but this stays dependency-free and predictable.
fn html_to_text(html: &str) -> String {
    let mut out = String::with_capacity(html.len() / 2);
    let mut rest = html;
    let mut skip_until: Option<&str> = None;

    loop {
        let lower = rest.to_lowercase();
        if let Some(tag) = skip_until {
            match lower.find(tag) {
                Some(index) => {
                    rest = &rest[index + tag.len()..];
                    skip_until = None;
                    continue;
                }
                None => break,
            }
        }

        match lower.find('<') {
            Some(index) => {
                out.push_str(&rest[..index]);
                let tail = &rest[index..];
                let Some(close) = tail.find('>') else { break };
                let tag = tail[..=close].to_lowercase();
                if tag.starts_with("<script") {
                    skip_until = Some("</script>");
                } else if tag.starts_with("<style") {
                    skip_until = Some("</style>");
                } else {
                    out.push(' ');
                }
                rest = &tail[close + 1..];
            }
            None => {
                out.push_str(rest);
                break;
            }
        }
    }

    out.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

async fn run_git(mut command: Command, ctx: &ToolContext) -> Result<String, String> {
    let output = command
        .current_dir(&ctx.workspace)
        .stdin(Stdio::null())
        .output()
        .await
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

async fn code_review(args: &Value, ctx: &ToolContext) -> ToolOutcome {
    let staged = args.get("staged").and_then(Value::as_bool).unwrap_or(false);
    let path = arg(args, "path").map(|value| resolve(&ctx.workspace, value));

    let mut stat = Command::new("git");
    stat.arg("diff").arg("--stat");
    let mut patch = Command::new("git");
    patch.arg("diff");
    if staged {
        stat.arg("--cached");
        patch.arg("--cached");
    }
    if let Some(target) = &path {
        stat.arg(target);
        patch.arg(target);
    }

    let stat_text = match run_git(stat, ctx).await {
        Ok(text) => text,
        Err(error) => return ToolOutcome::err(format!("code_review: {error}")),
    };
    let patch_text = match run_git(patch, ctx).await {
        Ok(text) => text,
        Err(error) => return ToolOutcome::err(format!("code_review: {error}")),
    };

    if patch_text.trim().is_empty() {
        return ToolOutcome::ok("(no changes to review in that scope)");
    }

    let body = format!("Summary:\n{stat_text}\nPatch:\n{patch_text}");
    ToolOutcome::ok(truncate(body, MAX_OUTPUT_CHARS))
}

async fn bash(args: &Value, ctx: &ToolContext) -> ToolOutcome {
    let Some(command) = arg(args, "command") else {
        return ToolOutcome::err("bash: missing 'command'");
    };
    let timeout_ms = args
        .get("timeout_ms")
        .and_then(Value::as_u64)
        .unwrap_or(120_000)
        .min(600_000);

    let mut shell = Command::new("sh");
    shell
        .arg("-c")
        .arg(command)
        .current_dir(&ctx.workspace)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    let child = match shell.spawn() {
        Ok(child) => child,
        Err(error) => return ToolOutcome::err(format!("bash: {error}")),
    };

    let output = match tokio::time::timeout(Duration::from_millis(timeout_ms), child.wait_with_output())
        .await
    {
        Ok(Ok(output)) => output,
        Ok(Err(error)) => return ToolOutcome::err(format!("bash: {error}")),
        Err(_) => return ToolOutcome::err(format!("bash: timed out after {timeout_ms} ms")),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let code = output
        .status
        .code()
        .map(|value| value.to_string())
        .unwrap_or_else(|| "signal".to_string());

    let mut result = format!("exit code: {code}\n");
    if !stdout.trim().is_empty() {
        result.push_str("stdout:\n");
        result.push_str(&stdout);
        result.push('\n');
    }
    if !stderr.trim().is_empty() {
        result.push_str("stderr:\n");
        result.push_str(&stderr);
        result.push('\n');
    }
    if stdout.trim().is_empty() && stderr.trim().is_empty() {
        result.push_str("(no output)\n");
    }

    let is_error = !output.status.success();
    let content = truncate(result, MAX_OUTPUT_CHARS);
    ToolOutcome {
        content,
        is_error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_to_text_drops_scripts_styles_and_tags() {
        let html = "<html><head><style>p{color:red}</style></head><body>\
                    <h1>Hallo</h1><script>var x = 1;</script><p>A&amp;B</p></body></html>";
        let text = html_to_text(html);
        assert!(text.contains("Hallo"), "kept heading text");
        assert!(text.contains("A&B"), "decoded entity: {text}");
        assert!(!text.contains("var x"), "dropped script body");
        assert!(!text.contains("color"), "dropped style body");
        assert!(!text.contains('<'), "dropped tags");
    }

    #[tokio::test]
    #[ignore = "requires network"]
    async fn live_web_fetch_reads_example_com() {
        let ctx = ToolContext {
            workspace: PathBuf::from("/tmp"),
            full_authorization: true,
            skills_root: PathBuf::from("/tmp"),
        };
        let mut todos = Vec::new();
        let outcome = execute(
            "web_fetch",
            &serde_json::json!({ "url": "https://example.com" }),
            &ctx,
            &mut todos,
        )
        .await;

        assert!(!outcome.is_error, "web_fetch failed: {}", outcome.content);
        assert!(
            outcome.content.to_lowercase().contains("example domain"),
            "unexpected body: {}",
            outcome.content
        );
        println!("bytes: {}", outcome.content.len());
    }

    #[tokio::test]
    #[ignore = "requires a git working tree"]
    async fn live_code_review_runs_git() {
        let workspace = std::env::current_dir().unwrap();
        let ctx = ToolContext {
            workspace,
            full_authorization: true,
            skills_root: PathBuf::from("/tmp"),
        };
        let mut todos = Vec::new();
        let outcome = execute("code_review", &serde_json::json!({}), &ctx, &mut todos).await;

        assert!(
            !outcome.is_error,
            "code_review failed: {}",
            outcome.content
        );
        println!("{}", outcome.content.lines().next().unwrap_or("(empty)"));
    }
}
