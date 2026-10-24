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

/// A capture that may be the window alone or, when the compositor would not say where the
/// window is, the whole desktop. `window` says which one arrived, so nobody has to guess.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowCapture {
    pub name: String,
    pub mime: String,
    pub data_base64: String,
    pub window: bool,
    pub note: String,
    /// Where the window was found, in desktop pixels, and how large the desktop is.
    pub rect: Option<[i64; 4]>,
    pub desktop: Option<[u32; 2]>,
}

/// Captures the whole virtual desktop and stores it next to the app data. The Composer puts
/// it into the normal attachment list, so the model sees exactly what the user sees.
#[tauri::command]
pub fn capture_screen(app: AppHandle) -> Result<ScreenCapture, String> {
    let bytes = capture_png(&app)?;
    Ok(ScreenCapture {
        name: capture_name(),
        mime: "image/png".to_string(),
        data_base64: crate::base64_encode(&bytes),
    })
}

/// The PNG bytes of the whole desktop; the temporary file belongs to the capture tool.
fn capture_png(app: &AppHandle) -> Result<Vec<u8>, String> {
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
    let path = directory.join(format!("capture-{stamp}.png"));

    capture_to(&path)?;
    let bytes = fs::read(&path).map_err(|error| format!("screenshot: {error}"))?;
    // Temporary: the attachment is written by the frontend, the model gets the bytes.
    let _ = fs::remove_file(&path);
    if bytes.len() < 64 {
        return Err("screenshot: the capture is empty".to_string());
    }
    if bytes.len() > 30 * 1024 * 1024 {
        return Err("screenshot: the capture is larger than 30 MB".to_string());
    }
    Ok(bytes)
}

fn capture_name() -> String {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    format!("screenshot-{stamp}.png")
}

/// Captures one of our own windows. A desktop tool can only photograph the screen, so the
/// window is cut out of the desktop capture afterwards: its position comes from the window
/// manager. A compositor that does not publish positions (some Wayland sessions) leaves the
/// full desktop as the honest fallback, with a note saying so.
#[tauri::command]
pub fn capture_window(app: AppHandle, id: String) -> Result<WindowCapture, String> {
    let window = app
        .get_webview_window(&id)
        .ok_or_else(|| format!("screenshot: no window '{id}'"))?;
    let bytes = capture_png(&app)?;

    let position = window.outer_position();
    let size = window.outer_size();
    let (Ok(position), Ok(size)) = (position, size) else {
        return Ok(WindowCapture {
            name: capture_name(),
            mime: "image/png".to_string(),
            data_base64: crate::base64_encode(&bytes),
            window: false,
            note: "the compositor does not publish window positions; this is the whole desktop"
                .to_string(),
            rect: None,
            desktop: None,
        });
    };
    if size.width == 0 || size.height == 0 {
        return Ok(WindowCapture {
            name: capture_name(),
            mime: "image/png".to_string(),
            data_base64: crate::base64_encode(&bytes),
            window: false,
            note: "the window has no size yet; this is the whole desktop".to_string(),
            rect: None,
            desktop: None,
        });
    }

    match crop_png(&bytes, position.x, position.y, size.width, size.height) {
        Ok((cropped, desktop)) => Ok(WindowCapture {
            name: capture_name(),
            mime: "image/png".to_string(),
            data_base64: crate::base64_encode(&cropped),
            window: true,
            note: format!("cropped to the window at {},{} ({}×{})", position.x, position.y, size.width, size.height),
            rect: Some([
                position.x as i64,
                position.y as i64,
                size.width as i64,
                size.height as i64,
            ]),
            desktop: Some(desktop),
        }),
        Err(error) => Ok(WindowCapture {
            name: capture_name(),
            mime: "image/png".to_string(),
            data_base64: crate::base64_encode(&bytes),
            window: false,
            note: format!("could not crop the window ({error}); this is the whole desktop"),
            rect: Some([
                position.x as i64,
                position.y as i64,
                size.width as i64,
                size.height as i64,
            ]),
            desktop: None,
        }),
    }
}

/// Cuts `width`×`height` at `x`,`y` out of a PNG. Coordinates are physical pixels, which is
/// what both the capture and the window reports use; the rectangle is clamped to the image so
/// a window partly off-screen still yields what is visible.
fn crop_png(
    png: &[u8],
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<(Vec<u8>, [u32; 2]), String> {
    let image = image::load_from_memory(png).map_err(|error| error.to_string())?;
    let desktop = [image.width(), image.height()];
    let left = x.max(0) as u32;
    let top = y.max(0) as u32;
    let right = (x + width as i32).clamp(0, desktop[0] as i32) as u32;
    let bottom = (y + height as i32).clamp(0, desktop[1] as i32) as u32;
    if right <= left || bottom <= top {
        return Err("the window is outside the captured screen".to_string());
    }
    let cropped = image.crop_imm(left, top, right - left, bottom - top);
    let mut out = Vec::new();
    cropped
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .map_err(|error| error.to_string())?;
    Ok((out, desktop))
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
