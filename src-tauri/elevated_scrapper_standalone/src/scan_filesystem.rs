use std::{fs, path::Path, str::FromStr};

use bluetooth_model::{BluetoothController, BluetoothDevice, BluetoothDeviceType};
use mac_address::MacAddress;

use crate::device_info::DeviceInfo;

pub fn scan_filesystem() -> Result<Vec<BluetoothController>, Box<dyn std::error::Error>> {
    let mut controllers = Vec::new();

    let base_path = Path::new("/var/lib/bluetooth/");

    for entry in fs::read_dir(base_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                eprintln!("Found controller: {}", name);
                // Attempt to scan controller files
                let controller = scan_controller_files(&path, name);

                match controller {
                    Ok(controller) => {
                        controllers.push(controller);
                    }
                    Err(e) => {
                        eprintln!("Failed to scan controller {}: {}", name, e);
                    }
                }
            }
        }
    }

    return Ok(controllers);
}

fn scan_controller_files(
    controller_path: &Path,
    file_name: &str,
) -> Result<BluetoothController, Box<dyn std::error::Error>> {
    let mac_address = MacAddress::from_str(file_name)?;

    let mut devices = Vec::new();

    // Scan for devices
    for entry in fs::read_dir(controller_path)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let device_address = MacAddress::from_str(
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default(),
        );
        if device_address.is_err() {
            eprintln!("Skipping non-device directory: {}", path.display());
            continue;
        }
        let device_address = device_address.unwrap();
        eprintln!("Found device: {}", device_address);

        // Device file will be `./info`
        let info_file = path.join("info");
        if info_file.exists() {
            if let Ok(device) = extract_device_details(&info_file, device_address) {
                devices.push(device);
            } else {
                eprintln!(
                    "Failed to extract device details from {}",
                    info_file.display()
                );
            }
        } else {
            eprintln!("No .info file found for device in {}", path.display());
        }
    }

    Ok(BluetoothController {
        name: Some(file_name.to_string()),
        address: mac_address,
        devices: devices,
    })
}

fn extract_device_details(
    device_path: &Path,
    address: MacAddress,
) -> Result<BluetoothDevice, Box<dyn std::error::Error>> {
    let device_info = DeviceInfo::load_from_file(device_path, address)?;

    let name = device_info.name();
    let link_key = device_info.link_key();
    let le_data = device_info.le_pairing_data();


    let device_type = match (link_key.is_some(), le_data.is_some()) {
        (true, true) => BluetoothDeviceType::DualMode,
        (true, false) => BluetoothDeviceType::Classic,
        (false, true) => BluetoothDeviceType::LowEnergy,
        (false, false) => BluetoothDeviceType::Corrupted, // Default fallback
    };

    Ok(BluetoothDevice {
        name,
        address,
        device_type,
        device_id: device_info.device_id(),
        link_key,
        le_data,
    })
}
