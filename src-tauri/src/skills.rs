use std::path::{Path, PathBuf};

use serde::Serialize;

/// Where skills live. Deliberately outside the app bundle so the user can drop files in
/// without rebuilding — the format is MiniMax-Code compatible (`<dir>/SKILL.md`).
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillSummary {
    pub name: String,
    pub description: String,
    pub path: String,
}

pub fn skills_root() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".ds-ide").join("skills")
}

pub fn ensure_root() -> PathBuf {
    let root = skills_root();
    let _ = std::fs::create_dir_all(&root);
    root
}

/// Read a `key: value` pair from YAML-ish front matter. Unknown structures are ignored on
/// purpose: a malformed skill should not take the whole list down.
fn front_matter_field(raw: &str, key: &str) -> Option<String> {
    let mut inside = false;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed == "---" {
            if inside {
                break;
            }
            inside = true;
            continue;
        }
        if !inside {
            continue;
        }
        if let Some(value) = trimmed.strip_prefix(&format!("{key}:")) {
            return Some(value.trim().trim_matches('"').trim().to_string());
        }
    }
    None
}

fn fallback_name(path: &Path) -> String {
    path.parent()
        .and_then(Path::file_name)
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn collect(paths: &mut Vec<PathBuf>, dir: &Path, depth: usize) {
    if depth > 3 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            collect(paths, &path, depth + 1);
            continue;
        }
        if entry.file_name().to_string_lossy().eq_ignore_ascii_case("SKILL.md") {
            paths.push(path);
        }
    }
}

fn skill_paths() -> Vec<PathBuf> {
    let root = skills_root();
    let mut paths = Vec::new();
    collect(&mut paths, &root, 0);
    paths
}

fn summarise(path: &Path) -> Option<SkillSummary> {
    let raw = std::fs::read_to_string(path).ok()?;
    Some(SkillSummary {
        name: front_matter_field(&raw, "name").unwrap_or_else(|| fallback_name(path)),
        description: front_matter_field(&raw, "description").unwrap_or_default(),
        path: path.to_string_lossy().to_string(),
    })
}

pub fn list() -> Vec<SkillSummary> {
    ensure_root();
    let mut skills: Vec<SkillSummary> = skill_paths().iter().filter_map(|path| summarise(path)).collect();
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    skills
}

pub fn find(name: &str) -> Option<PathBuf> {
    for path in skill_paths() {
        let matches = summarise(&path)
            .map(|summary| summary.name == name)
            .unwrap_or(false);
        if matches {
            return Some(path);
        }
    }
    None
}

#[tauri::command]
pub fn skills_list() -> Vec<SkillSummary> {
    list()
}

#[tauri::command]
pub fn skills_root_path() -> String {
    ensure_root().to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_front_matter_and_falls_back_to_directory_name() {
        let raw = "---\nname: landing-page-builder\ndescription: Build a landing page\n---\n\n# Body\n";
        assert_eq!(
            front_matter_field(raw, "name").as_deref(),
            Some("landing-page-builder")
        );
        assert_eq!(
            front_matter_field(raw, "description").as_deref(),
            Some("Build a landing page")
        );

        let without_name = "---\ndescription: no name here\n---\n";
        assert_eq!(front_matter_field(without_name, "name"), None);
    }

    /// Depends on the user's own ~/.ds-ide/skills, so it is opt-in.
    #[test]
    #[ignore = "depends on the user's ~/.ds-ide/skills"]
    fn list_local_skills_smoke() {
        let found = list();
        println!("{} skill(s) in {}", found.len(), skills_root().display());
        for skill in &found {
            println!("  {} — {}", skill.name, skill.description);
        }
    }
}
