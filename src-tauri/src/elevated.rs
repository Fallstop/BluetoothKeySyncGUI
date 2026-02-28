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
        .arg("--")
        .arg(program)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run pkexec: {}", e))
}
