use bluetooth_model::batch::{
    BatchOperation, BatchOperationResult, BatchRequest, BatchResponse,
};
use bluetooth_model::worker_protocol::{
    BatchItemResult, WorkerCommand, WorkerOperation, WorkerReady, WorkerResponse,
    WorkerResponseData, WorkerResult,
};
use bluetooth_model::{BluetoothData, BluetoothDevice, HostDistributions};

mod device_info;
mod scan_filesystem;

use base64::Engine;
use clap::{Parser, Subcommand};
use device_info::DeviceInfo;
use mac_address::MacAddress;
use std::io::{BufRead, Write};
use std::path::Path;
use std::str::FromStr;

/// Interface for elevated scrapper standalone
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,

    #[arg(short, long, global = true)]
    privileged: bool,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Scan and output all BT data as JSON (existing behavior)
    Scan,

    /// Write keys to a device's info file
    WriteKeys {
        /// Controller MAC address (AA:BB:CC:DD:EE:FF)
        #[arg(long)]
        controller: String,

        /// Device MAC address (AA:BB:CC:DD:EE:FF)
        #[arg(long)]
        device: String,

        /// Base64-encoded JSON of BluetoothDevice
        #[arg(long)]
        data: String,
    },

    /// Delete a device directory
    DeleteDevice {
        /// Controller MAC address (AA:BB:CC:DD:EE:FF)
        #[arg(long)]
        controller: String,

        /// Device MAC address (AA:BB:CC:DD:EE:FF)
        #[arg(long)]
        device: String,
    },

    /// Restart the bluetoothd service so synced keys take effect
    RestartBluetooth,

    /// Execute multiple operations in a single invocation (one password prompt)
    Batch {
        /// Base64-encoded JSON of BatchRequest
        #[arg(long)]
        data: String,
    },

    /// Run as a persistent worker, reading JSON commands from stdin
    Serve,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    #[cfg(not(feature = "assume_elevated"))]
    if !args.privileged {
        // Makes it easy to develop without needing to run it explicitly as root
        sudo::escalate_if_needed()?;
    }

    match args.command {
        Command::Scan => {
            let data = read_bluetooth_data()?;
            let json_output = serde_json::to_string(&data)?;
            print!("{}", json_output);
        }
        Command::WriteKeys {
            controller,
            device,
            data,
        } => {
            let result = handle_write_keys_b64(&controller, &device, &data);
            print_result(result);
        }
        Command::DeleteDevice {
            controller,
            device,
        } => {
            let result = handle_delete_device(&controller, &device);
            print_result(result);
        }
        Command::RestartBluetooth => {
            let result = handle_restart_bluetooth();
            print_result(result);
        }
        Command::Batch { data } => {
            let response = handle_batch(&data);
            let json = serde_json::to_string(&response)
                .unwrap_or_else(|e| format!("{{\"results\":[],\"error\":\"{}\"}}", e));
            print!("{}", json);
        }
        Command::Serve => {
            serve_loop()?;
        }
    }

    Ok(())
}

fn read_bluetooth_data() -> Result<BluetoothData, Box<dyn std::error::Error>> {
    let controllers = scan_filesystem::scan_filesystem()?;
    Ok(BluetoothData {
        host: HostDistributions::Linux,
        controllers,
        utc_timestamp: chrono::Utc::now(),
        source_path: "/var/lib/bluetooth/".to_string(),
    })
}

/// Write keys from a base64-encoded JSON string (CLI path).
fn handle_write_keys_b64(
    controller: &str,
    device: &str,
    data_b64: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let json_bytes = base64::engine::general_purpose::STANDARD.decode(data_b64)?;
    let source_device: BluetoothDevice = serde_json::from_slice(&json_bytes)?;
    write_keys_to_device(controller, device, &source_device)
}

/// Write keys from a raw JSON string (serve path).
fn handle_write_keys_json(
    controller: &str,
    device: &str,
    data_json: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let source_device: BluetoothDevice = serde_json::from_str(data_json)?;
    write_keys_to_device(controller, device, &source_device)
}

/// Core key-writing logic shared by both CLI and serve paths.
fn write_keys_to_device(
    controller: &str,
    device: &str,
    source_device: &BluetoothDevice,
) -> Result<(), Box<dyn std::error::Error>> {
    let controller_mac = MacAddress::from_str(controller)?;
    let device_mac = MacAddress::from_str(device)?;

    let device_dir = format!(
        "/var/lib/bluetooth/{}/{}",
        controller_mac, device_mac
    );
    let info_path = format!("{}/info", device_dir);

    // Ensure directory exists
    std::fs::create_dir_all(&device_dir)?;

    // Set restrictive permissions on the device directory
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&device_dir, std::fs::Permissions::from_mode(0o700))?;
    }

    // Load existing info file or create new one
    let mut device_info = if Path::new(&info_path).exists() {
        DeviceInfo::load_from_file(&info_path, device_mac)?
    } else {
        let mut info = DeviceInfo::new(device_mac);
        // Set name from incoming data if creating new
        if let Some(ref name) = source_device.name {
            info.set_name(Some(name.clone()));
        }
        info
    };

    // Write link key if present
    if source_device.link_key.is_some() {
        device_info.set_link_key(source_device.link_key.clone());
    }

    // Write LE data if present
    if let Some(ref le_data) = source_device.le_data {
        device_info.set_le_pairing_data(le_data.clone());
    }

    device_info.save_to_file(&info_path)?;

    eprintln!("Successfully wrote keys to {}", info_path);
    Ok(())
}

fn handle_delete_device(
    controller: &str,
    device: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let controller_mac = MacAddress::from_str(controller)?;
    let device_mac = MacAddress::from_str(device)?;

    let device_dir = format!(
        "/var/lib/bluetooth/{}/{}",
        controller_mac, device_mac
    );

    let device_dir_path = Path::new(&device_dir);
    if device_dir_path.exists() {
        // Defense-in-depth: verify path is under /var/lib/bluetooth/
        let canonical = std::fs::canonicalize(device_dir_path)?;
        if !canonical.starts_with("/var/lib/bluetooth/") {
            return Err(format!(
                "Refusing to delete path outside /var/lib/bluetooth/: {}",
                canonical.display()
            )
            .into());
        }
        std::fs::remove_dir_all(&canonical)?;
        eprintln!("Successfully deleted {}", device_dir);
    } else {
        eprintln!("Device directory does not exist: {}", device_dir);
    }

    Ok(())
}

fn handle_restart_bluetooth() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::process::Command::new("systemctl")
        .args(["restart", "bluetooth"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to restart bluetooth service: {}", stderr).into());
    }

    eprintln!("Successfully restarted bluetooth service");

    // After a service restart, BlueZ brings the adapter up powered-off,
    // which makes KDE show Bluetooth as "disabled" until manually re-enabled.
    // Spawn a background thread to power it back on after the service settles,
    // so we don't block the worker response.
    std::thread::spawn(|| {
        // Wait for systemd to report the service as active
        for i in 0..10 {
            std::thread::sleep(std::time::Duration::from_millis(500));
            let status = std::process::Command::new("systemctl")
                .args(["is-active", "bluetooth"])
                .output();
            if let Ok(out) = status {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if stdout.trim() == "active" {
                    eprintln!("bluetooth service active after {}ms", (i + 1) * 500);
                    break;
                }
            }
        }

        // Additional settle time for BlueZ to finish adapter initialization
        std::thread::sleep(std::time::Duration::from_secs(2));

        for attempt in 1..=3 {
            let power_output = std::process::Command::new("bluetoothctl")
                .args(["power", "on"])
                .output();

            match power_output {
                Ok(out) => {
                    let stdout_str = String::from_utf8_lossy(&out.stdout);
                    eprintln!(
                        "bluetoothctl power on attempt {}: exit={}, stdout={}",
                        attempt, out.status, stdout_str.trim(),
                    );

                    if let Ok(show) = std::process::Command::new("bluetoothctl")
                        .args(["show"])
                        .output()
                    {
                        let show_out = String::from_utf8_lossy(&show.stdout);
                        if show_out.contains("Powered: yes") {
                            eprintln!("Adapter confirmed powered on");
                            return;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: could not run bluetoothctl (attempt {}): {}", attempt, e);
                }
            }

            if attempt < 3 {
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
        }

        eprintln!("Warning: adapter may not be powered on after all retries");
    });

    Ok(())
}

fn handle_batch(data_b64: &str) -> BatchResponse {
    let json_bytes = match base64::engine::general_purpose::STANDARD.decode(data_b64) {
        Ok(b) => b,
        Err(e) => {
            return BatchResponse {
                results: vec![BatchOperationResult {
                    index: 0,
                    success: false,
                    error: Some(format!("Failed to decode base64: {}", e)),
                }],
                scan_data: None,
            };
        }
    };

    let request: BatchRequest = match serde_json::from_slice(&json_bytes) {
        Ok(r) => r,
        Err(e) => {
            return BatchResponse {
                results: vec![BatchOperationResult {
                    index: 0,
                    success: false,
                    error: Some(format!("Failed to parse BatchRequest JSON: {}", e)),
                }],
                scan_data: None,
            };
        }
    };

    let mut results = Vec::with_capacity(request.operations.len());
    let mut scan_data = None;

    for (index, op) in request.operations.iter().enumerate() {
        match op {
            BatchOperation::WriteKeys {
                controller,
                device,
                data,
            } => {
                let result = handle_write_keys_b64(controller, device, data);
                results.push(BatchOperationResult {
                    index,
                    success: result.is_ok(),
                    error: result.err().map(|e| e.to_string()),
                });
            }
            BatchOperation::DeleteDevice {
                controller,
                device,
            } => {
                let result = handle_delete_device(controller, device);
                results.push(BatchOperationResult {
                    index,
                    success: result.is_ok(),
                    error: result.err().map(|e| e.to_string()),
                });
            }
            BatchOperation::RestartBluetooth => {
                let result = handle_restart_bluetooth();
                results.push(BatchOperationResult {
                    index,
                    success: result.is_ok(),
                    error: result.err().map(|e| e.to_string()),
                });
            }
            BatchOperation::Scan => match read_bluetooth_data() {
                Ok(data) => {
                    scan_data = Some(data);
                    results.push(BatchOperationResult {
                        index,
                        success: true,
                        error: None,
                    });
                }
                Err(e) => {
                    results.push(BatchOperationResult {
                        index,
                        success: false,
                        error: Some(e.to_string()),
                    });
                }
            },
        }
    }

    BatchResponse { results, scan_data }
}

// --- Serve mode: persistent worker loop ---

fn serve_loop() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = std::io::stdout();
    let stdin = std::io::stdin();
    let mut stdout_lock = stdout.lock();

    // Send ready signal
    let ready = WorkerReady { ready: true };
    serde_json::to_writer(&mut stdout_lock, &ready)?;
    stdout_lock.write_all(b"\n")?;
    stdout_lock.flush()?;

    // Read commands line-by-line
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to read stdin: {}", e);
                break;
            }
        };

        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        let cmd: WorkerCommand = match serde_json::from_str(&line) {
            Ok(c) => c,
            Err(e) => {
                // Can't parse command — write an error response with id 0
                let resp = WorkerResponse {
                    id: 0,
                    result: WorkerResult::Err {
                        message: format!("Failed to parse command: {}", e),
                    },
                };
                serde_json::to_writer(&mut stdout_lock, &resp)?;
                stdout_lock.write_all(b"\n")?;
                stdout_lock.flush()?;
                continue;
            }
        };

        // Shutdown requested
        if matches!(cmd.op, WorkerOperation::Shutdown) {
            let resp = WorkerResponse {
                id: cmd.id,
                result: WorkerResult::Ok { data: None },
            };
            serde_json::to_writer(&mut stdout_lock, &resp)?;
            stdout_lock.write_all(b"\n")?;
            stdout_lock.flush()?;
            break;
        }

        let result = dispatch_command(&cmd.op);
        let resp = WorkerResponse {
            id: cmd.id,
            result,
        };
        serde_json::to_writer(&mut stdout_lock, &resp)?;
        stdout_lock.write_all(b"\n")?;
        stdout_lock.flush()?;
    }

    Ok(())
}

fn dispatch_command(op: &WorkerOperation) -> WorkerResult {
    match op {
        WorkerOperation::Scan => match read_bluetooth_data() {
            Ok(data) => WorkerResult::Ok {
                data: Some(WorkerResponseData::ScanResult {
                    bluetooth_data: data,
                }),
            },
            Err(e) => WorkerResult::Err {
                message: e.to_string(),
            },
        },
        WorkerOperation::WriteKeys {
            controller,
            device,
            data,
        } => match handle_write_keys_json(controller, device, data) {
            Ok(()) => WorkerResult::Ok { data: None },
            Err(e) => WorkerResult::Err {
                message: e.to_string(),
            },
        },
        WorkerOperation::DeleteDevice {
            controller,
            device,
        } => match handle_delete_device(controller, device) {
            Ok(()) => WorkerResult::Ok { data: None },
            Err(e) => WorkerResult::Err {
                message: e.to_string(),
            },
        },
        WorkerOperation::RestartBluetooth => match handle_restart_bluetooth() {
            Ok(()) => WorkerResult::Ok { data: None },
            Err(e) => WorkerResult::Err {
                message: e.to_string(),
            },
        },
        WorkerOperation::Batch { operations } => dispatch_batch(operations),
        WorkerOperation::Ping => WorkerResult::Ok {
            data: Some(WorkerResponseData::Pong),
        },
        WorkerOperation::Shutdown => {
            // Handled in serve_loop before dispatch
            WorkerResult::Ok { data: None }
        }
    }
}

fn dispatch_batch(operations: &[WorkerOperation]) -> WorkerResult {
    let mut results = Vec::with_capacity(operations.len());
    let mut scan_data = None;

    for (index, op) in operations.iter().enumerate() {
        match op {
            WorkerOperation::WriteKeys {
                controller,
                device,
                data,
            } => {
                let result = handle_write_keys_json(controller, device, data);
                results.push(BatchItemResult {
                    index,
                    success: result.is_ok(),
                    error: result.err().map(|e| e.to_string()),
                });
            }
            WorkerOperation::DeleteDevice {
                controller,
                device,
            } => {
                let result = handle_delete_device(controller, device);
                results.push(BatchItemResult {
                    index,
                    success: result.is_ok(),
                    error: result.err().map(|e| e.to_string()),
                });
            }
            WorkerOperation::RestartBluetooth => {
                let result = handle_restart_bluetooth();
                results.push(BatchItemResult {
                    index,
                    success: result.is_ok(),
                    error: result.err().map(|e| e.to_string()),
                });
            }
            WorkerOperation::Scan => match read_bluetooth_data() {
                Ok(data) => {
                    scan_data = Some(data);
                    results.push(BatchItemResult {
                        index,
                        success: true,
                        error: None,
                    });
                }
                Err(e) => {
                    results.push(BatchItemResult {
                        index,
                        success: false,
                        error: Some(e.to_string()),
                    });
                }
            },
            // Nested batch or other ops inside batch are not supported
            _ => {
                results.push(BatchItemResult {
                    index,
                    success: false,
                    error: Some("Unsupported operation in batch".to_string()),
                });
            }
        }
    }

    WorkerResult::Ok {
        data: Some(WorkerResponseData::BatchResult {
            results,
            scan_data,
        }),
    }
}

fn print_result(result: Result<(), Box<dyn std::error::Error>>) {
    match result {
        Ok(_) => {
            print!("{{\"success\":true}}");
        }
        Err(e) => {
            let error_json = serde_json::json!({
                "success": false,
                "error": e.to_string()
            });
            print!("{}", error_json);
        }
    }
}
