use std::io::Write;
use std::path::PathBuf;
use std::process::{Command as StdCommand, Stdio};

use tauri::utils::platform;

pub fn relative_command_path(command: &str) -> Result<PathBuf, String> {
    let exe = platform::current_exe()
        .map_err(|e| format!("Failed to get current executable path: {}", e))?;
    match exe.parent() {
        #[cfg(windows)]
        Some(exe_dir) => Ok(exe_dir.join(command).with_extension("exe")),
        #[cfg(not(windows))]
        Some(exe_dir) => Ok(exe_dir.join(command)),
        None => Err("Executable has no parent directory".to_string()),
    }
}

pub fn is_elevated() -> bool {
    unsafe { libc::getuid() == 0 }
}

/// Check if running inside a Flatpak sandbox.
pub fn is_flatpak() -> bool {
    std::env::var("FLATPAK_ID").is_ok()
}

/// Check if running inside a Snap sandbox.
pub fn is_snap() -> bool {
    std::env::var("SNAP").is_ok()
}

/// Check if running inside an AppImage.
/// The AppImage runtime sets the `APPIMAGE` environment variable.
pub fn is_appimage() -> bool {
    std::env::var("APPIMAGE").is_ok()
}

/// Copy the elevated worker binary out of the AppImage FUSE mount to a real
/// filesystem path that root can access. The AppImage FUSE mount at
/// `/tmp/.mount_*` is only visible to the mounting user, so pkexec/sudo
/// running as root cannot access it.
///
/// Copies to `/tmp/bluetooth-key-sync-elevated_scrapper` with mode 0755.
/// Returns the path to the copied binary.
pub fn appimage_extract_binary(source: &std::path::Path) -> Result<PathBuf, String> {
    let dest = PathBuf::from("/tmp/bluetooth-key-sync-elevated_scrapper");

    std::fs::copy(source, &dest).map_err(|e| {
        format!(
            "AppImage: failed to copy elevated binary from {} to {}: {}",
            source.display(),
            dest.display(),
            e
        )
    })?;

    // Ensure the copy is executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o755)).map_err(|e| {
            format!("AppImage: failed to set permissions on {}: {}", dest.display(), e)
        })?;
    }

    Ok(dest)
}

/// Clean up the temporary elevated binary extracted for AppImage.
pub fn appimage_cleanup_binary() {
    let dest = std::path::Path::new("/tmp/bluetooth-key-sync-elevated_scrapper");
    if dest.exists() {
        let _ = std::fs::remove_file(dest);
    }
}

/// Resolve the host-visible path to the elevated scrapper binary when running
/// inside a Flatpak sandbox. Uses `flatpak-spawn --host flatpak info --show-location`
/// to find the Flatpak installation path, then constructs the path to the binary
/// inside the app's `files/bin/` directory.
pub fn flatpak_host_binary_path(binary_name: &str) -> Result<PathBuf, String> {
    let output = StdCommand::new("flatpak-spawn")
        .args([
            "--host",
            "flatpak",
            "info",
            "--show-location",
            "nz.jmw.bluetooth-key-sync",
        ])
        .output()
        .map_err(|e| format!("Failed to query Flatpak install location: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("flatpak info failed: {}", stderr.trim()));
    }

    let location = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if location.is_empty() {
        return Err("Flatpak install location is empty".to_string());
    }

    Ok(PathBuf::from(location)
        .join("files")
        .join("bin")
        .join(binary_name))
}

/// Find a graphical askpass program.
pub fn find_askpass() -> Option<String> {
    // Check SSH_ASKPASS environment variable first (standard mechanism)
    if let Ok(askpass) = std::env::var("SSH_ASKPASS") {
        if !askpass.is_empty() {
            if std::path::Path::new(&askpass).exists() {
                return Some(askpass);
            }
            // Try resolving via which if not an absolute path
            if let Ok(output) = StdCommand::new("which").arg(&askpass).output() {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path.is_empty() {
                        return Some(path);
                    }
                }
            }
        }
    }

    for name in &["ksshaskpass", "ssh-askpass"] {
        if let Ok(output) = StdCommand::new("which").arg(name).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Some(path);
                }
            }
        }
    }
    None
}

/// Run a command with root privileges using a graphical password prompt + sudo -S.
/// Falls back to pkexec if no askpass program is found.
pub fn run_elevated(program: &PathBuf, args: &[&str]) -> Result<std::process::Output, String> {
    // Try graphical askpass + sudo -S (works with sudo-rs on Kubuntu/Wayland)
    if let Some(askpass) = find_askpass() {
        let askpass_output = StdCommand::new(&askpass)
            .arg("Bluetooth Key Sync needs root access to read Bluetooth keys")
            .output()
            .map_err(|e| format!("Failed to run askpass ({}): {}", askpass, e))?;

        if !askpass_output.status.success() {
            return Err("Authentication cancelled".to_string());
        }

        let mut child = StdCommand::new("sudo")
            .arg("-S")
            .arg("--")
            .arg(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn sudo: {}", e))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(&askpass_output.stdout)
                .map_err(|e| format!("Failed to write password to sudo: {}", e))?;
            stdin
                .write_all(b"\n")
                .map_err(|e| format!("Failed to write newline to sudo: {}", e))?;
        }

        return child
            .wait_with_output()
            .map_err(|e| format!("Failed to wait for sudo: {}", e));
    }

    // Fallback to pkexec (works on X11 / traditional polkit setups)
    StdCommand::new("pkexec")
        .arg(program)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run pkexec: {}", e))
}
