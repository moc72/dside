use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Manager};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenCapture {
    pub name: String,
    pub mime: String,
    pub data_base64: String,
}

/// Captures the whole virtual desktop and stores it next to the app data. The Composer puts
/// it into the normal attachment list, so the model sees exactly what the user sees.
#[tauri::command]
pub fn capture_screen(app: AppHandle) -> Result<ScreenCapture, String> {
    let directory = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("screenshots");
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    let name = format!("screenshot-{stamp}.png");
    let path = directory.join(&name);

    capture_to(&path)?;
    let bytes = fs::read(&path).map_err(|error| format!("screenshot: {error}"))?;
    if bytes.len() < 64 {
        return Err("screenshot: the capture is empty".to_string());
    }
    if bytes.len() > 30 * 1024 * 1024 {
        return Err("screenshot: the capture is larger than 30 MB".to_string());
    }

    // The compressed attachment is written by the frontend; the full capture is only a
    // temporary file for the native screenshot tool.
    let _ = fs::remove_file(&path);

    Ok(ScreenCapture {
        name,
        mime: "image/png".to_string(),
        data_base64: crate::base64_encode(&bytes),
    })
}

#[cfg(windows)]
fn capture_to(path: &Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;

    let escaped = path.to_string_lossy().replace('\'', "''");
    let script = format!(
        "Add-Type -AssemblyName System.Windows.Forms; \
         Add-Type -AssemblyName System.Drawing; \
         $b=[System.Windows.Forms.SystemInformation]::VirtualScreen; \
         $bitmap=New-Object System.Drawing.Bitmap $b.Width,$b.Height; \
         $graphics=[System.Drawing.Graphics]::FromImage($bitmap); \
         $graphics.CopyFromScreen($b.Location,[System.Drawing.Point]::Empty,$b.Size); \
         $bitmap.Save('{escaped}',[System.Drawing.Imaging.ImageFormat]::Png); \
         $graphics.Dispose(); $bitmap.Dispose()"
    );

    let mut command = Command::new("powershell.exe");
    command.args([
        "-NoProfile",
        "-NonInteractive",
        "-WindowStyle",
        "Hidden",
        "-Command",
        &script,
    ]);
    command.creation_flags(0x0800_0000);
    let output = command
        .output()
        .map_err(|error| format!("screenshot: powershell: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "screenshot: powershell: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    if !path.is_file() {
        return Err("screenshot: no file was written".to_string());
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn capture_to(path: &Path) -> Result<(), String> {
    let output = Command::new("/usr/sbin/screencapture")
        .args(["-x", "-t", "png"])
        .arg(path)
        .output()
        .map_err(|error| format!("screenshot: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "screenshot: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    if !path.is_file() {
        return Err("screenshot: no file was written (screen recording permission?)".to_string());
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn capture_to(path: &Path) -> Result<(), String> {
    use std::process::Stdio;

    fn attempt(program: &str, args: &[&str], path: &Path) -> bool {
        // A capture tool that cannot work here (grim on a compositor without the screencopy
        // protocol, for example) is expected: the next one is tried. Its stderr would land in
        // the app's terminal and claim a failure that did not happen — the screenshot was
        // taken by the tool after it. Only the first successful tool is worth a log line.
        let Ok(status) = Command::new(program)
            .args(args)
            .arg(path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
        else {
            return false;
        };
        status.success()
            && path.is_file()
            && fs::metadata(path).map(|meta| meta.len() > 0).unwrap_or(false)
    }

    // Wayland first, then the common X11/GNOME/KDE tools. Every one writes to the given path.
    for (program, args) in [
        ("grim", &[][..]),
        ("gnome-screenshot", &["-f"][..]),
        ("spectacle", &["-b", "-n", "-o"][..]),
        ("scrot", &[][..]),
        ("import", &["-window", "root"][..]),
    ] {
        if attempt(program, args, path) {
            crate::log_line!("screenshot: captured with {program}");
            return Ok(());
        }
    }

    Err(
        "screenshot: no capture tool found (grim, gnome-screenshot, spectacle, scrot or ImageMagick)"
            .to_string(),
    )
}
