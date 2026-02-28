use bluetooth_model::{BluetoothData, BluetoothDevice, HostDistributions};

mod device_info;
mod scan_filesystem;

use base64::Engine;
use clap::{Parser, Subcommand};
use device_info::DeviceInfo;
use mac_address::MacAddress;
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
            let result = handle_write_keys(&controller, &device, &data);
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

fn handle_write_keys(
    controller: &str,
    device: &str,
    data_b64: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Decode base64 -> JSON -> BluetoothDevice
    let json_bytes = base64::engine::general_purpose::STANDARD.decode(data_b64)?;
    let source_device: BluetoothDevice = serde_json::from_slice(&json_bytes)?;

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
        device_info.set_link_key(source_device.link_key);
    }

    // Write LE data if present
    if let Some(le_data) = source_device.le_data {
        device_info.set_le_pairing_data(le_data);
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
    Ok(())
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
