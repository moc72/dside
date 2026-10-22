use std::fs;
use std::path::Path;
use std::process::Command;

use serde::Serialize;

/// A Git panel without a Git library: the CLI is already a dependency of the code-review
/// tool and behaves the same on Windows, macOS and Linux. Every call uses `--` before paths
/// and never goes through a shell.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitEntry {
    pub path: String,
    pub code: String,
    pub staged: bool,
    pub untracked: bool,
    pub conflicted: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitStatus {
    pub branch: String,
    pub head: String,
    pub entries: Vec<GitEntry>,
}

fn run_git(workspace: &str, args: &[&str]) -> Result<Vec<u8>, String> {
    if !Path::new(workspace).is_dir() {
        return Err(format!("Arbeitsverzeichnis nicht gefunden: {workspace}"));
    }

    let mut command = Command::new("git");
    command
        .current_dir(workspace)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_PAGER", "cat");

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // No console flash when a GUI app shells out to git.
        command.creation_flags(0x0800_0000);
    }

    let output = command
        .output()
        .map_err(|error| format!("git konnte nicht gestartet werden: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Err(if stderr.is_empty() { stdout } else { stderr });
    }
    Ok(output.stdout)
}

fn git_text(workspace: &str, args: &[&str]) -> Option<String> {
    run_git(workspace, args)
        .ok()
        .map(|bytes| String::from_utf8_lossy(&bytes).trim().to_string())
}

/// Parses `git status --porcelain=v1 -z`. A file with both staged and unstaged edits yields
/// two entries, so the panel can show both sides and diff exactly the side that was clicked.
fn parse_status(bytes: &[u8]) -> Vec<GitEntry> {
    let mut entries = Vec::new();
    let mut parts = bytes.split(|byte| *byte == 0);

    while let Some(raw) = parts.next() {
        if raw.is_empty() || raw.len() < 4 {
            continue;
        }
        let xy = &raw[..2];
        let path = String::from_utf8_lossy(&raw[3..]).to_string();
        let x = xy[0] as char;
        let y = xy[1] as char;
        let untracked = xy == b"??";
        let conflicted = x == 'U' || y == 'U' || xy == b"AA" || xy == b"DD";
        let staged = x != ' ' && x != '?';
        let unstaged = y != ' ' && y != '?';

        if staged {
            entries.push(GitEntry {
                path: path.clone(),
                code: x.to_string(),
                staged: true,
                untracked: false,
                conflicted,
            });
        }
        if unstaged {
            entries.push(GitEntry {
                path: path.clone(),
                code: y.to_string(),
                staged: false,
                untracked: false,
                conflicted,
            });
        }
        if untracked {
            entries.push(GitEntry {
                path: path.clone(),
                code: "??".to_string(),
                staged: false,
                untracked: true,
                conflicted: false,
            });
        }

        // Rename/copy entries carry the original path in the following NUL segment.
        if x == 'R' || x == 'C' || y == 'R' || y == 'C' {
            let _ = parts.next();
        }
    }

    entries
}

#[tauri::command]
pub fn git_panel_status(workspace: String) -> Result<GitStatus, String> {
    let bytes = run_git(
        &workspace,
        &[
            "--no-pager",
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
        ],
    )?;
    let entries = parse_status(&bytes);
    let branch = git_text(&workspace, &["--no-pager", "branch", "--show-current"])
        .unwrap_or_default();
    let head = git_text(&workspace, &["--no-pager", "rev-parse", "--short", "HEAD"])
        .unwrap_or_default();
    Ok(GitStatus {
        branch,
        head,
        entries,
    })
}

#[tauri::command]
pub fn git_panel_diff(
    workspace: String,
    path: String,
    staged: bool,
    untracked: bool,
) -> Result<String, String> {
    if untracked {
        let full = Path::new(&workspace).join(&path);
        let bytes = fs::read(&full).map_err(|error| format!("Datei nicht lesbar: {error}"))?;
        if bytes.contains(&0) {
            return Ok("Binärdatei — keine Textvorschau.".to_string());
        }
        let text = String::from_utf8_lossy(&bytes);
        let mut diff = format!("--- /dev/null\n+++ b/{path}\n");
        for line in text.lines() {
            diff.push('+');
            diff.push_str(line);
            diff.push('\n');
        }
        return Ok(limit_diff(diff));
    }

    let mut args = vec!["--no-pager", "diff", "--no-color"];
    if staged {
        args.push("--cached");
    }
    args.push("--");
    args.push(&path);
    let bytes = run_git(&workspace, &args)?;
    Ok(limit_diff(String::from_utf8_lossy(&bytes).to_string()))
}

/// A huge generated file should not freeze the webview; the panel says what was cut.
fn limit_diff(mut diff: String) -> String {
    const LIMIT: usize = 400_000;
    if diff.len() > LIMIT {
        let mut cut = LIMIT;
        while cut > 0 && !diff.is_char_boundary(cut) {
            cut -= 1;
        }
        diff.truncate(cut);
        diff.push_str("\n… Diff gekürzt …\n");
    }
    diff
}

#[tauri::command]
pub fn git_panel_stage(workspace: String, paths: Vec<String>) -> Result<(), String> {
    if paths.is_empty() {
        return Ok(());
    }
    let mut args = vec!["add", "--"];
    args.extend(paths.iter().map(String::as_str));
    run_git(&workspace, &args)?;
    Ok(())
}

#[tauri::command]
pub fn git_panel_unstage(workspace: String, paths: Vec<String>) -> Result<(), String> {
    if paths.is_empty() {
        return Ok(());
    }
    let mut args = vec!["--no-pager", "reset", "-q", "HEAD", "--"];
    args.extend(paths.iter().map(String::as_str));
    run_git(&workspace, &args)?;
    Ok(())
}

#[tauri::command]
pub fn git_panel_stage_all(workspace: String) -> Result<(), String> {
    run_git(&workspace, &["add", "-A"])?;
    Ok(())
}

#[tauri::command]
pub fn git_panel_commit(workspace: String, message: String) -> Result<String, String> {
    let message = message.trim();
    if message.is_empty() {
        return Err("Die Commit-Nachricht ist leer.".to_string());
    }
    let bytes = run_git(&workspace, &["--no-pager", "commit", "-m", message])?;
    Ok(String::from_utf8_lossy(&bytes).trim().to_string())
}

#[tauri::command]
pub fn git_panel_branches(workspace: String) -> Result<Vec<String>, String> {
    let bytes = run_git(
        &workspace,
        &[
            "--no-pager",
            "for-each-ref",
            "--format=%(refname:short)",
            "refs/heads",
        ],
    )?;
    Ok(String::from_utf8_lossy(&bytes)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect())
}

#[tauri::command]
pub fn git_panel_checkout(workspace: String, branch: String) -> Result<(), String> {
    let branch = branch.trim();
    if branch.is_empty() || branch.starts_with('-') {
        return Err("Ungültiger Zweig.".to_string());
    }
    run_git(&workspace, &["--no-pager", "checkout", branch])?;
    Ok(())
}

#[tauri::command]
pub fn git_panel_pull(workspace: String) -> Result<String, String> {
    let bytes = run_git(&workspace, &["--no-pager", "pull", "--ff-only"])?;
    let text = String::from_utf8_lossy(&bytes).trim().to_string();
    if text.is_empty() {
        Ok("Bereits aktuell.".to_string())
    } else {
        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_staged_unstaged_untracked_renamed_and_conflicted() {
        let raw =
            b" M a.txt\0?? new.txt\0MM both.txt\0R  new-name.txt\0old-name.txt\0UU conflict.txt\0";
        let entries = parse_status(raw);

        assert_eq!(entries.len(), 7);
        assert_eq!(entries[0].path, "a.txt");
        assert!(!entries[0].staged);
        assert_eq!(entries[0].code, "M");

        assert_eq!(entries[1].path, "new.txt");
        assert!(entries[1].untracked);

        assert_eq!(entries[2].path, "both.txt");
        assert!(entries[2].staged);
        assert_eq!(entries[3].path, "both.txt");
        assert!(!entries[3].staged);

        assert_eq!(entries[4].path, "new-name.txt");
        assert!(entries[4].staged);
        assert_eq!(entries[4].code, "R");

        assert_eq!(entries[5].path, "conflict.txt");
        assert!(entries[5].conflicted);
        assert_eq!(entries[6].path, "conflict.txt");
        assert!(entries[6].conflicted);
        assert!(!entries[6].staged);
    }

    #[test]
    fn rejects_short_or_empty_entries() {
        assert!(parse_status(b"").is_empty());
        assert!(parse_status(b"abc\0").is_empty());
    }

    /// Exercises the real git binary against a throwaway repository: status, stage, diff and
    /// commit. Ignored by default because CI images and developer machines must have git.
    #[test]
    #[ignore = "requires the git executable"]
    fn live_git_panel_round_trip() {
        let directory =
            std::env::temp_dir().join(format!("ds-ide-git-panel-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();

        let git = |args: &[&str]| {
            let status = Command::new("git")
                .current_dir(&directory)
                .args(args)
                .status()
                .expect("run git");
            assert!(status.success(), "git {:?} failed", args);
        };
        git(&["init", "-q"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);

        fs::write(directory.join("a.txt"), "eins\n").unwrap();
        let workspace = directory.to_string_lossy().to_string();

        let status = git_panel_status(workspace.clone()).unwrap();
        assert!(status.entries.iter().any(|entry| entry.path == "a.txt" && entry.untracked));

        git_panel_stage(workspace.clone(), vec!["a.txt".to_string()]).unwrap();
        let status = git_panel_status(workspace.clone()).unwrap();
        assert!(status.entries.iter().any(|entry| entry.path == "a.txt" && entry.staged));

        let diff = git_panel_diff(workspace.clone(), "a.txt".to_string(), true, false).unwrap();
        assert!(diff.contains("+eins"));

        git_panel_commit(workspace.clone(), "test commit".to_string()).unwrap();
        let status = git_panel_status(workspace.clone()).unwrap();
        assert!(status.entries.is_empty(), "working tree should be clean");

        git(&["branch", "other"]);
        let branches = git_panel_branches(workspace.clone()).unwrap();
        assert!(branches.iter().any(|branch| branch == "other"));
        git_panel_checkout(workspace.clone(), "other".to_string()).unwrap();
        let status = git_panel_status(workspace.clone()).unwrap();
        assert_eq!(status.branch, "other");

        let _ = fs::remove_dir_all(&directory);
    }
}
