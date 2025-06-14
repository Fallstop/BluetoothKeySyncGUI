use std::{error::Error, path::Path, str::FromStr};
use chrono::Utc;
use mac_address::MacAddress;
use nt_hive2::*;
use std::fs::File; // Changed from tokio::fs::File
use tokio::task; // Added for spawn_blocking
use crate::bluetooth::bluetooth_data::{BluetoothController, BluetoothData, BluetoothDevice, BluetoothDeviceType, BluetoothLowEnergyKey, HostDistributions};

pub async fn extract_hive_data(hive_path: &Path) -> Result<BluetoothData, Box<dyn std::error::Error + Send + Sync>> {
    let path_buf = hive_path.to_path_buf();

    // Use spawn_blocking to run the blocking code in a separate thread
    let bluetooth_data = task::spawn_blocking(move || -> Result<BluetoothData, Box<dyn std::error::Error + Send + Sync>> {
        let hive_file = File::open(path_buf)?;
        let hive_result = Hive::new(hive_file, HiveParseMode::NormalWithBaseBlock);
        if let Err(e) = hive_result {
          let message = format!("Failed to open hive file: {}", e);
          return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, message)));
        }
        let mut hive = hive_result.unwrap();

        let root_key = hive.root_key_node()?;
				// Heavily based on this: https://unix.stackexchange.com/a/255510

				// Let's find the bluetooth info, but iterating through possible paths
				let key_paths = [
					r"ControlSet001\Services\BTHPORT\Parameters\Keys",
					r"ControlSet002\Services\BTHPORT\Parameters\Keys",
					r"CurrentControlSet\Services\BTHPORT\Parameters\Keys",
					r"ControlSet001\services\BTHPORT\Parameters\Keys",
					r"ControlSet002\services\BTHPORT\Parameters\Keys",
					r"CurrentControlSet\services\BTHPORT\Parameters\Keys",
				];


				for key_path in key_paths.iter() {
					if let Ok(key) = root_key.subpath(*key_path, &mut hive) {
						if key.is_none() {
							continue; // Skip if the key does not exist
						}
						let key = key.unwrap();

						let mut controllers = Vec::new();

						for subkey in key.borrow().subkeys(&mut hive)?.iter() {
							let controller_address = subkey.borrow().name().to_string();
							let mut devices = Vec::new();

							println!("Found controller: {}", controller_address);
							println!("It has {} devices connected", subkey.borrow().subkey_count() as usize + subkey.borrow().values().len());

							// Iterate through the subkeys of the controller.
							// These are the LE devices connected to the controller, I think??
							for device_connected in (*subkey.borrow().subkeys(&mut hive)?).iter() {
								let device_key = device_connected.borrow();
								let device_name = device_key.name().to_string();
								let device_info = device_key.values();

								// https://unix.stackexchange.com/a/413831

								let get_value = |name: &str| {
									device_info.iter().find(|x| x.name() == name).map(|x| x.value().to_string())
								};

								let le_data = BluetoothLowEnergyKey {
									identity_resolving_key: get_value("IRK"),
									local_signature_key: get_value("CSRK"),
									long_term_key: get_value("LTK"),
									rand: get_value("ERand"),
									ediv: get_value("EDIV"),
								};



								devices.push(BluetoothDevice {
									name: Some(device_name.clone()),
									address: MacAddress::from_str(&device_name).unwrap_or(MacAddress::default()),
									device_type: BluetoothDeviceType::LowEnergy,
									link_key: None,
									le_data: Some(le_data)
								});
							}

							// Now let's hunt for the classic devices connected to the controller.
							for device_connected in subkey.borrow().values() {
								// Skip if the name is not a valid MAC address length
								if device_connected.name().len() != 12 {
									println!("Skipping extra key: {}", device_connected.name());
									continue;
								}

								let device_name = device_connected.name().to_string();
								let device_value = device_connected.value().to_string();

								devices.push(BluetoothDevice {
									name: Some(device_name.clone()),
									address: MacAddress::from_str(&device_name).unwrap_or(MacAddress::default()),
									device_type: BluetoothDeviceType::Classic,
									link_key: Some(device_value),
									le_data: None,
								});
							}


							let controller = BluetoothController {
								name: subkey.borrow().name().to_string().into(),
								address: MacAddress::from_str(&controller_address).unwrap_or(MacAddress::default()),
								devices,
							};
							controllers.push(controller);

						}
						return Ok(BluetoothData {
							host: HostDistributions::Windows,
							controllers: controllers,
							utc_timestamp: Utc::now(),
						});
					}
				}
        Ok(BluetoothData::default())
    }).await??;

    Ok(bluetooth_data)
}
