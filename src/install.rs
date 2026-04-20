use std::fs;
use std::path::PathBuf;

#[cfg(not(feature = "dev"))]
const EXTENSION_ID: &str = "ggbcdjkfblmmccmcemnaaiomnfjphpbd";

#[cfg(feature = "dev")]
const EXTENSION_ID: &str = "efagbebpfhdjdcehgaabblmdceamcofb";
const HOST_NAME: &str = "com.clickpoint";

pub fn install() {
    match do_install() {
        Ok(paths) => {
            let list = paths.join("\n");
            println!("Manifest saved to:\n{}", list);
            show_message("Click Point", "Installation completed", false);
        }
        Err(e) => {
            show_message("Click Point — installation error", &e.to_string(), true);
        }
    }
}

fn do_install() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let exe_path = std::env::current_exe()?.canonicalize()?;
    let exe_str = exe_path
        .to_str()
        .ok_or("Installation path contains incorrect symbols")?;

    let manifest = generate_manifest(exe_str);

    #[cfg(not(windows))]
    let dirs = get_native_messaging_dirs()?;

    let mut saved_paths = Vec::new();

    #[cfg(not(windows))]
    for dir in &dirs {
        fs::create_dir_all(dir)?;
        let file_path = dir.join(format!("{}.json", HOST_NAME));
        fs::write(&file_path, &manifest)?;
        saved_paths.push(file_path.to_string_lossy().into_owned());
    }

    // On Windows: one manifest file next to the exe, registry keys for all browsers.
    #[cfg(windows)]
    {
        let manifest_path = exe_path
            .parent()
            .ok_or("Cannot get a directory with binary file")?
            .join(format!("{}.json", HOST_NAME));
        fs::write(&manifest_path, &manifest)?;
        saved_paths.push(manifest_path.to_string_lossy().into_owned());
        register_windows_registry(&manifest_path)?;
    }

    Ok(saved_paths)
}

fn generate_manifest(exe_path: &str) -> String {
    let manifest = serde_json::json!({
        "name": HOST_NAME,
        "description": "Move mouse to (x, y)",
        "path": exe_path,
        "type": "stdio",
        "allowed_origins": [
            format!("chrome-extension://{}/", EXTENSION_ID)
        ]
    });
    serde_json::to_string_pretty(&manifest).unwrap()
}

// ── Platform-specific: directories ──────────────────────────────────────────

#[cfg(target_os = "linux")]
fn get_native_messaging_dirs() -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let home = std::env::var("HOME")?;
    let base = PathBuf::from(&home).join(".config");

    let browser_dirs = [
        "google-chrome",
        "google-chrome-beta",
        "google-chrome-unstable",
        "chromium",
        "BraveSoftware/Brave-Browser",
        "BraveSoftware/Brave-Browser-Beta",
        "microsoft-edge",
        "microsoft-edge-beta",
        "vivaldi",
        "opera",
        "opera-beta",
        "yandex-browser",
    ];

    Ok(browser_dirs
        .iter()
        .map(|d| base.join(d).join("NativeMessagingHosts"))
        .collect())
}

#[cfg(target_os = "macos")]
fn get_native_messaging_dirs() -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let home = std::env::var("HOME")?;
    let base = PathBuf::from(&home).join("Library/Application Support");

    let browser_dirs = [
        "Google/Chrome",
        "Google/Chrome Beta",
        "Google/Chrome Canary",
        "Chromium",
        "BraveSoftware/Brave-Browser",
        "BraveSoftware/Brave-Browser-Beta",
        "Microsoft Edge",
        "Microsoft Edge Beta",
        "Vivaldi",
        "com.operasoftware.Opera",
        "Yandex/YandexBrowser",
    ];

    Ok(browser_dirs
        .iter()
        .map(|d| base.join(d).join("NativeMessagingHosts"))
        .collect())
}

#[cfg(windows)]
fn get_native_messaging_dirs() -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let appdata = std::env::var("APPDATA")?;
    let base = PathBuf::from(&appdata);

    let browser_dirs = [
        r"Google\Chrome\User Data",
        r"Chromium\User Data",
        r"BraveSoftware\Brave-Browser\User Data",
        r"Microsoft\Edge\User Data",
        r"Vivaldi\User Data",
        r"Opera Software\Opera Stable",
    ];

    Ok(browser_dirs
        .iter()
        .map(|d| base.join(d).join("NativeMessagingHosts"))
        .collect())
}

// ── Windows: registry ────────────────────────────────────────────────────────

#[cfg(windows)]
fn register_windows_registry(
    manifest_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    use winreg::RegKey;
    use winreg::enums::*;

    let manifest_str = manifest_path.to_str().ok_or("Incorrect path to manifest")?;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    let registry_paths = [
        format!(r"Software\Google\Chrome\NativeMessagingHosts\{}", HOST_NAME),
        format!(r"Software\Chromium\NativeMessagingHosts\{}", HOST_NAME),
        format!(
            r"Software\BraveSoftware\Brave-Browser\NativeMessagingHosts\{}",
            HOST_NAME
        ),
        format!(
            r"Software\Microsoft\Edge\NativeMessagingHosts\{}",
            HOST_NAME
        ),
        format!(r"Software\Vivaldi\NativeMessagingHosts\{}", HOST_NAME),
        format!(
            r"Software\Opera Software\Opera Stable\NativeMessagingHosts\{}",
            HOST_NAME
        ),
    ];

    for key_path in &registry_paths {
        if let Ok((key, _)) = hkcu.create_subkey(key_path) {
            let _ = key.set_value("", &manifest_str);
        }
    }

    Ok(())
}

// ── Platform-specific: GUI notification ─────────────────────────────────────

#[cfg(target_os = "linux")]
fn show_message(title: &str, message: &str, is_error: bool) {
    let icon = if is_error { "error" } else { "info" };

    // Try notify-send (desktop notification — no window needed)
    let ok = std::process::Command::new("notify-send")
        .args(["--icon", icon, "--app-name", "Click Point", title, message])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if ok {
        return;
    }

    // Fallback: zenity dialog
    let dialog_type = if is_error { "--error" } else { "--info" };
    let ok = std::process::Command::new("zenity")
        .args([
            dialog_type,
            "--title",
            title,
            "--text",
            message,
            "--no-markup",
        ])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if ok {
        return;
    }

    // Last resort: terminal output
    if is_error {
        eprintln!("{}: {}", title, message);
    } else {
        println!("{}: {}", title, message);
    }
    println!("Press Enter to exit...");
    let mut buf = String::new();
    let _ = std::io::stdin().read_line(&mut buf);
}

#[cfg(target_os = "macos")]
fn show_message(title: &str, message: &str, _is_error: bool) {
    let script = format!(
        "display dialog \"{}\" with title \"{}\" buttons {{\"OK\"}} default button \"OK\"",
        message.replace('"', "\\\""),
        title.replace('"', "\\\""),
    );
    let _ = std::process::Command::new("osascript")
        .args(["-e", &script])
        .status();
}

#[cfg(windows)]
fn show_message(title: &str, message: &str, is_error: bool) {
    let icon = if is_error { 16 } else { 64 }; // MB_ICONERROR | MB_ICONINFORMATION
    let script = format!(
        "Add-Type -AssemblyName System.Windows.Forms; \
         [System.Windows.Forms.MessageBox]::Show('{}', '{}', 'OK', {});",
        message.replace('\'', "''"),
        title.replace('\'', "''"),
        icon,
    );
    let _ = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .status();
}
