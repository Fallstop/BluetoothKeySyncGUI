use std::io::Write;
use std::path::PathBuf;
use std::process::{Command as StdCommand, Stdio};

use bluetooth_model::BluetoothData;
use tauri::utils::platform;

use crate::api::message::Message;

fn relative_command_path(command: &str) -> PathBuf {
    match platform::current_exe().unwrap().parent() {
        #[cfg(windows)]
        Some(exe_dir) => exe_dir.join(command).with_extension("exe"),
        #[cfg(not(windows))]
        Some(exe_dir) => exe_dir.join(command),
        None => unimplemented!(),
    }
}

fn is_elevated() -> bool {
    unsafe { libc::getuid() == 0 }
}

/// Find a graphical askpass program (KDE's ksshaskpass or generic ssh-askpass).
fn find_askpass() -> Option<String> {
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
fn run_elevated(program: &PathBuf, args: &[&str]) -> Result<std::process::Output, String> {
    // Try graphical askpass + sudo -S (works with sudo-rs on Kubuntu/Wayland)
    if let Some(askpass) = find_askpass() {
        println!("Using askpass: {}", askpass);
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
            let _ = stdin.write_all(&askpass_output.stdout);
            let _ = stdin.write_all(b"\n");
            // stdin is dropped here, closing the pipe
        }

        return child
            .wait_with_output()
            .map_err(|e| format!("Failed to wait for sudo: {}", e));
    }

    // Fallback to pkexec (works on X11 / traditional polkit setups)
    println!("No askpass found, falling back to pkexec");
    StdCommand::new("pkexec")
        .arg(program)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run pkexec: {}", e))
}

#[taurpc::procedures(path = "linux")]
pub trait LinuxApi {
    async fn parse_local_config() -> Message<BluetoothData>;
}

#[derive(Clone)]
pub struct LinuxApiImpl;

#[taurpc::resolvers]
impl LinuxApi for LinuxApiImpl {
    async fn parse_local_config(self) -> Message<BluetoothData> {
        let path = relative_command_path("elevated_scrapper");

        println!(
            "Running command: {:?} scan --privileged (elevated: {})",
            path,
            is_elevated()
        );

        let output = if is_elevated() {
            StdCommand::new(&path)
                .arg("scan")
                .arg("--privileged")
                .output()
                .map_err(|e| format!("Failed to run elevated scrapper: {}", e))
        } else {
            run_elevated(&path, &["scan", "--privileged"])
        };

        let output = match output {
            Ok(o) => o,
            Err(e) => return Message::Error(e),
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Message::Error(format!(
                "Elevated scrapper exited with {}: {}",
                output.status, stderr
            ));
        }

        println!(
            "Output received: {} bytes stdout, {} bytes stderr",
            output.stdout.len(),
            output.stderr.len()
        );

        match serde_json::from_slice::<BluetoothData>(&output.stdout) {
            Ok(data) => Message::Success(data),
            Err(e) => Message::Error(format!("Failed to parse Bluetooth data: {}", e)),
        }
    }
}
