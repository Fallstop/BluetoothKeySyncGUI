use std::{fs, path::Path, str::FromStr};

use bluetooth_model::BluetoothController;
use mac_address::MacAddress;

pub fn scan_filesystem() -> Result<Vec<BluetoothController>, Box<dyn std::error::Error>> {
    let mut controllers = Vec::new();

    let base_path = Path::new("/var/lib/bluetooth/");

    for entry in fs::read_dir(base_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                println!("Found controller: {}", name);
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

    // Scan for devices
    for entry in fs::read_dir(controller_path)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {continue;}

        // Attempt to find device files
        for entry in fs::read_dir(controller_path)? {
            let entry = entry?;
            let device_path = entry.path();

        }

    }


    Ok(BluetoothController {
        name: Some(file_name.to_string()),
        address: mac_address,
        devices: Vec::new(),
    })
}
