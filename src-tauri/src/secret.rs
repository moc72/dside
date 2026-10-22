use std::path::PathBuf;

use serde::Serialize;

const SERVICE: &str = "ds-ide.deepvibe";
const ACCOUNT: &str = "deepseek-api-key";

/// Where the API key currently lives. Surfaced to the UI so the fallback is never silent.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretStatus {
    pub present: bool,
    /// "keyring" | "file" | "none"
    pub backend: String,
}

pub struct SecretStore {
    fallback_path: PathBuf,
}

impl SecretStore {
    pub fn new(fallback_path: PathBuf) -> Self {
        Self { fallback_path }
    }

    fn entry() -> Option<keyring::Entry> {
        keyring::Entry::new(SERVICE, ACCOUNT).ok()
    }

    /// Keyring first, then the 0600 fallback file. Never a plaintext file without a warning.
    pub fn get(&self) -> Option<String> {
        if let Some(entry) = Self::entry() {
            if let Ok(value) = entry.get_password() {
                if !value.trim().is_empty() {
                    return Some(value);
                }
            }
        }
        self.read_fallback()
    }

    pub fn set(&self, value: &str) -> SecretStatus {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return self.clear();
        }

        if let Some(entry) = Self::entry() {
            if entry.set_password(trimmed).is_ok() {
                // `set_password` can succeed while the value lives only inside the entry
                // that wrote it — keyring's mock store does exactly that, and a missing
                // platform feature is enough to select it. Read the value back through a
                // *fresh* entry, which is what the next request will do.
                let verified = match keyring::Entry::new(SERVICE, ACCOUNT) {
                    Ok(fresh) => match fresh.get_password() {
                        Ok(stored) if stored == trimmed => true,
                        Ok(_) => {
                            // A wrong value must not win over the file fallback.
                            let _ = fresh.delete_credential();
                            false
                        }
                        // A transient read failure keeps the entry; the file still works.
                        Err(_) => false,
                    },
                    Err(_) => false,
                };

                if verified {
                    // Keyring won: make sure no stale copy stays behind on disk.
                    let _ = std::fs::remove_file(&self.fallback_path);
                    return SecretStatus {
                        present: true,
                        backend: "keyring".to_string(),
                    };
                }
            }
        }

        self.write_fallback(trimmed);
        SecretStatus {
            present: true,
            backend: "file".to_string(),
        }
    }

    pub fn clear(&self) -> SecretStatus {
        if let Some(entry) = Self::entry() {
            let _ = entry.delete_credential();
        }
        let _ = std::fs::remove_file(&self.fallback_path);
        SecretStatus {
            present: false,
            backend: "none".to_string(),
        }
    }

    pub fn status(&self) -> SecretStatus {
        if let Some(entry) = Self::entry() {
            if entry
                .get_password()
                .map(|value| !value.trim().is_empty())
                .unwrap_or(false)
            {
                return SecretStatus {
                    present: true,
                    backend: "keyring".to_string(),
                };
            }
        }
        if self.read_fallback().is_some() {
            return SecretStatus {
                present: true,
                backend: "file".to_string(),
            };
        }
        SecretStatus {
            present: false,
            backend: "none".to_string(),
        }
    }

    fn read_fallback(&self) -> Option<String> {
        let raw = std::fs::read_to_string(&self.fallback_path).ok()?;
        let parsed: serde_json::Value = serde_json::from_str(&raw).ok()?;
        parsed
            .get("apiKey")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
    }

    fn write_fallback(&self, value: &str) {
        if let Some(parent) = self.fallback_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let body = serde_json::json!({ "apiKey": value }).to_string();
        if std::fs::write(&self.fallback_path, body).is_ok() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(
                    &self.fallback_path,
                    std::fs::Permissions::from_mode(0o600),
                );
            }
        }
    }
}

#[tauri::command]
pub fn secret_status(store: tauri::State<'_, SecretStore>) -> SecretStatus {
    store.status()
}

#[tauri::command]
pub fn secret_set(store: tauri::State<'_, SecretStore>, value: String) -> SecretStatus {
    store.set(&value)
}

#[tauri::command]
pub fn secret_clear(store: tauri::State<'_, SecretStore>) -> SecretStatus {
    store.clear()
}

#[cfg(test)]
mod tests {
    /// Proves the OS keyring backend actually works on this machine. Skips (passes)
    /// where no keyring is reachable, so it is safe in a bare environment.
    #[test]
    fn keyring_roundtrip_or_skip() {
        let Ok(entry) = keyring::Entry::new("ds-ide.test", "roundtrip") else {
            crate::log_line!("keyring: no backend available — skipped");
            return;
        };
        if let Err(error) = entry.set_password("probe") {
            crate::log_line!("keyring: write failed ({error}) — skipped");
            return;
        }
        // Read through a second entry. A backend that keeps the value only inside the
        // entry (keyring's mock store, which is used when no platform feature applies)
        // passes the same-entry check and loses the value as soon as the entry drops.
        let reader = keyring::Entry::new("ds-ide.test", "roundtrip").expect("second entry");
        assert_eq!(reader.get_password().unwrap(), "probe");
        let _ = entry.delete_credential();
    }
}
