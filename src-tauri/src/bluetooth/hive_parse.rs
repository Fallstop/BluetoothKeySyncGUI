use bluetooth_model::{
    BluetoothController, BluetoothData, BluetoothDevice, BluetoothDeviceType, BluetoothLinkKey, BluetoothLowEnergyKey, HostDistributions
};
use chrono::Utc;
use mac_address::MacAddress;
use nt_hive2::*;
use std::collections::HashMap;
use std::fs::File;
use std::{path::Path, str::FromStr};
use tokio::task;

pub async fn extract_hive_data(
    hive_path: &Path,
) -> Result<BluetoothData, Box<dyn std::error::Error + Send + Sync>> {
    let hive_path_str = hive_path.to_string_lossy().to_string();
    let path_buf = hive_path.to_path_buf();

    let bluetooth_data = task::spawn_blocking(
        move || -> Result<BluetoothData, Box<dyn std::error::Error + Send + Sync>> {
            let hive_file = File::open(path_buf)?;
            let hive_result = Hive::new(hive_file, HiveParseMode::NormalWithBaseBlock);
            if let Err(e) = hive_result {
                let message = format!("Failed to open hive file: {}", e);
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    message,
                )));
            }
            let mut hive = hive_result.unwrap();
            let root_key = hive.root_key_node()?;

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
                        continue;
                    }
                    let key = key.unwrap();
                    let mut controllers = Vec::new();

                    for subkey in key.borrow().subkeys(&mut hive)?.iter() {
                        let controller_address = subkey.borrow().name().to_string();

                        println!("Found controller: {}", controller_address);
                        println!(
                            "It has {} devices connected",
                            subkey.borrow().subkey_count() as usize
                                + subkey.borrow().values().len()
                        );

                        let devices = parse_controller_devices(&subkey.borrow(), &root_key, &mut hive)?;

                        let controller = BluetoothController {
                            name: None,
                            address: MacAddress::from_str(&controller_address)
                                .unwrap_or(MacAddress::default()),
                            devices,
                        };
                        controllers.push(controller);
                    }

                    return Ok(BluetoothData {
                        host: HostDistributions::Windows,
                        controllers,
                        utc_timestamp: Utc::now(),
                        source_path: hive_path_str,
                    });
                }
            }
            Ok(BluetoothData::default())
        },
    )
    .await??;

    Ok(bluetooth_data)
}

fn parse_controller_devices(
    controller_key: &KeyNode,
    root_key: &KeyNode,
    hive: &mut Hive<File, CleanHive>,
) -> Result<Vec<BluetoothDevice>, Box<dyn std::error::Error + Send + Sync>> {
    let mut device_map: HashMap<String, (Option<BluetoothLinkKey>, Option<BluetoothLowEnergyKey>)> = HashMap::new();

    // First pass: collect LE devices (subkeys)
    for device_connected in (*controller_key.subkeys(hive)?).iter() {
        let device_key = device_connected.borrow();
        let device_mac = device_key.name().to_string();
        let device_info = device_key.values();

        let get_value = |name: &str| {
            device_info
                .iter()
                .find(|x| x.name() == name)
                .map(|x| x.value().to_string())
        };

        let le_data = BluetoothLowEnergyKey {
            identity_resolving_key: get_value("IRK"),
            local_signature_key: get_value("CSRK"),
            long_term_key: get_value("LTK"),
            key_length: get_value("KeyLength").and_then(|s| s.parse().ok()),
            rand: get_value("ERand"),
            ediv: get_value("EDIV"),
        };

        device_map.insert(device_mac, (None, Some(le_data)));
    }

    // Second pass: collect Classic devices (values)
    for device_connected in controller_key.values() {
        if device_connected.name().len() != 12 {
            println!("Skipping extra key: {}", device_connected.name());
            continue;
        }

        let device_mac = device_connected.name().to_string();
        let link_key_string = device_connected.value().to_string();
        let link_key = if link_key_string.is_empty() {
            None
        } else {
            Some(BluetoothLinkKey {
                key: link_key_string,
            })
        };

        if let Some(entry) = device_map.get_mut(&device_mac) {
            // Device exists in both modes - make it dual mode
            entry.0 = link_key;
        } else {
            // Classic-only device
            device_map.insert(device_mac, (link_key, None));
        }
    }

    // Third pass: create BluetoothDevice objects
    let mut devices = Vec::new();
    for (device_mac, (link_key, le_data)) in device_map {
        let device_name = get_device_name_from_cache(root_key, &device_mac, hive)
            .unwrap_or_else(|| device_mac.clone());

        let device_type = match (&link_key, &le_data) {
            (Some(_), Some(_)) => BluetoothDeviceType::DualMode,
            (Some(_), None) => BluetoothDeviceType::Classic,
            (None, Some(_)) => BluetoothDeviceType::LowEnergy,
            (None, None) => continue, // Skip invalid entries
        };

        devices.push(BluetoothDevice {
            name: Some(device_name),
            address: MacAddress::from_str(&device_mac).unwrap_or(MacAddress::default()),
            device_id: None,
            device_type,
            link_key,
            le_data,
        });
    }

    Ok(devices)
}

fn get_device_name_from_cache(
    root_key: &KeyNode,
    mac_address: &str,
    hive: &mut Hive<File, CleanHive>,
) -> Option<String> {
    // Also has a whole bunch of extra data! Will be useful later.
    // (...)\BTHPORT\Parameters\Devices\4801c54b1bfd> ls
    // Node has 3 subkeys and 24 values
    //   key name
    //   <CachedServices>
    //   <DynamicCachedServices>
    //   <ServicesForb07d6414f31a>
    //   size     type              value name             [value if type DWORD]
    //      4  4 REG_DWORD          <COD>                5898764 [0x5a020c]
    //      4  4 REG_DWORD          <DibServiceVersion>   131072 [0x20000]
    //    298  1 REG_SZ             <FingerprintString>
    //      1  3 REG_BINARY         <FriendlyName>
    //      8  b REG_QWORD          <HostSupportedFeaturesMap>
    //      8  b REG_QWORD          <LastConnected>
    //      8  b REG_QWORD          <LastSeen>
    //      4  4 REG_DWORD          <LEAddressType>            0 [0x0]
    //      4  4 REG_DWORD          <LEAppearance>             0 [0x0]
    //     16  3 REG_BINARY         <LeContainerId>
    //      4  4 REG_DWORD          <LeContainerIDSource>      3 [0x3]
    //      8  b REG_QWORD          <LMPFeatures>
    //      4  4 REG_DWORD          <LmpSubversion>          702 [0x2be]
    //      4  4 REG_DWORD          <LmpVersion>               9 [0x9]
    //      4  4 REG_DWORD          <LocalEvaldIoCap>          1 [0x1]
    //      4  4 REG_DWORD          <LocalEvaldIoCapLE>      255 [0xff]
    //      4  4 REG_DWORD          <ManufacturerId>          29 [0x1d]
    //     10  3 REG_BINARY         <Name>
    //      4  4 REG_DWORD          <PID>                   4608 [0x1200]
    //      4  4 REG_DWORD          <Version>                  0 [0x0]
    //      4  4 REG_DWORD          <VID>                    224 [0xe0]
    //      4  4 REG_DWORD          <VIDType>                  1 [0x1]
    //      8  b REG_QWORD          <FingerprintTimestamp>
    //      4  4 REG_DWORD          <FingerprintVersion>       3 [0x3]

    let device_cache_paths = [
        r"ControlSet001\Services\BTHPORT\Parameters\Devices",
        r"ControlSet002\Services\BTHPORT\Parameters\Devices",
        r"CurrentControlSet\Services\BTHPORT\Parameters\Devices",
    ];

    // nightmarish...

    for cache_path in device_cache_paths.iter() {
        if let Ok(Some(devices_key)) = root_key.subpath(*cache_path, hive) {
            if let Ok(device_subkeys) = devices_key.borrow().subkeys(hive) {
                for device_subkey in device_subkeys.iter() {
                    let device_key = device_subkey.borrow();
                    if device_key.name().to_lowercase() == mac_address.to_lowercase() {
                        for value in device_key.values() {
                            if value.name() == "Name" || value.name() == "FriendlyName"  || value.name() == "LEName"{
                                if let Some(name) = extract_string(value.value()) {
                                    return Some(name);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}


fn extract_string(reg: &RegistryValue) -> Option<String> {
    match reg {
        RegistryValue::RegSZ(s) => Some(s.trim().to_owned()),
        RegistryValue::RegExpandSZ(s) => Some(s.trim().to_owned()),
        RegistryValue::RegBinary(b) => {
            if let Ok(s) = String::from_utf8(b.to_vec()) {
                Some(s.trim().to_owned())
            } else {
                None
            }
        },
        RegistryValue::RegMultiSZ(s) => Some(s.join("\n")),
        _ => None,
    }
}
