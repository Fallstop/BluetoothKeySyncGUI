use bluetooth_model::{
    BluetoothController, BluetoothData, BluetoothDevice, BluetoothDeviceType, BluetoothLinkKey,
    BluetoothLowEnergyKey, LongTermKeyData, SignatureKeyData, HostDistributions,
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
) -> std::result::Result<BluetoothData, Box<dyn std::error::Error + Send + Sync>> {
    let hive_path_str = hive_path.to_string_lossy().to_string();
    let path_buf = hive_path.to_path_buf();

    let bluetooth_data = task::spawn_blocking(
        move || -> std::result::Result<BluetoothData, Box<dyn std::error::Error + Send + Sync>> {
            let hive_file = File::open(path_buf)?;
            let mut hive = Hive::new(hive_file, HiveParseMode::NormalWithBaseBlock)
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to open hive file: {}", e),
                    ))
                })?;
            let root_key = hive.root_key_node()
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;

            let bthport_paths = [
                r"ControlSet001\Services\BTHPORT\Parameters\Keys",
                r"ControlSet002\Services\BTHPORT\Parameters\Keys",
                r"CurrentControlSet\Services\BTHPORT\Parameters\Keys",
                r"ControlSet001\services\BTHPORT\Parameters\Keys",
                r"ControlSet002\services\BTHPORT\Parameters\Keys",
                r"CurrentControlSet\services\BTHPORT\Parameters\Keys",
            ];

            // Maps controller MAC -> HashMap<device MAC -> (link_key, le_data)>
            let mut controller_devices: HashMap<String, HashMap<String, (Option<BluetoothLinkKey>, Option<BluetoothLowEnergyKey>)>> = HashMap::new();

            // Scan BTHPORT (Classic + some LE devices)
            for key_path in bthport_paths.iter() {
                if let Ok(Some(key)) = root_key.subpath(*key_path, &mut hive) {
                    for subkey in key.borrow().subkeys(&mut hive)
                        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?.iter() {
                        let controller_address = subkey.borrow().name().to_string();

                        println!("Found BTHPORT controller: {}", controller_address);

                        let devices = parse_controller_devices(&subkey.borrow(), &root_key, &mut hive)?;
                        let device_map = controller_devices.entry(controller_address).or_default();
                        for dev in devices {
                            let mac = dev.address.to_string().replace(":", "").to_uppercase();
                            device_map.insert(mac, (dev.link_key, dev.le_data));
                        }
                    }
                    break; // Use first found ControlSet
                }
            }

            // Scan BTHLE (pure-LE devices not under BTHPORT)
            let bthle_paths = [
                r"ControlSet001\Services\BTHLE\Parameters\Keys",
                r"ControlSet002\Services\BTHLE\Parameters\Keys",
                r"CurrentControlSet\Services\BTHLE\Parameters\Keys",
                r"ControlSet001\services\BTHLE\Parameters\Keys",
                r"ControlSet002\services\BTHLE\Parameters\Keys",
                r"CurrentControlSet\services\BTHLE\Parameters\Keys",
            ];

            for key_path in bthle_paths.iter() {
                if let Ok(Some(key)) = root_key.subpath(*key_path, &mut hive) {
                    for adapter_subkey in key.borrow().subkeys(&mut hive)
                        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?.iter() {
                        let controller_address = adapter_subkey.borrow().name().to_string();
                        println!("Found BTHLE controller: {}", controller_address);

                        let device_map = controller_devices.entry(controller_address).or_default();

                        for device_subkey in adapter_subkey.borrow().subkeys(&mut hive)
                            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?.iter() {
                            let device_key = device_subkey.borrow();
                            let device_mac = device_key.name().to_string();
                            let device_info = device_key.values();

                            let get_binary_hex = |name: &str| -> Option<String> {
                                device_info
                                    .iter()
                                    .find(|x| x.name() == name)
                                    .and_then(|x| match x.value() {
                                        RegistryValue::RegBinary(bytes) => {
                                            Some(bytes.iter().map(|b| format!("{:02X}", b)).collect())
                                        }
                                        _ => None,
                                    })
                            };

                            let get_dword = |name: &str| -> Option<u32> {
                                device_info
                                    .iter()
                                    .find(|x| x.name() == name)
                                    .and_then(|x| match x.value() {
                                        RegistryValue::RegDWord(v) => Some(*v),
                                        _ => None,
                                    })
                            };

                            let get_qword_string = |name: &str| -> Option<String> {
                                device_info
                                    .iter()
                                    .find(|x| x.name() == name)
                                    .and_then(|x| match x.value() {
                                        RegistryValue::RegQWord(v) => Some(v.to_string()),
                                        _ => None,
                                    })
                            };

                            if let Some(le_data) = parse_le_values(&get_binary_hex, &get_dword, &get_qword_string) {
                                let entry = device_map.entry(device_mac).or_insert((None, None));
                                if entry.1.is_none() {
                                    entry.1 = Some(le_data);
                                }
                            }
                        }
                    }
                    break; // Use first found ControlSet
                }
            }

            if controller_devices.is_empty() {
                return Ok(BluetoothData::default());
            }

            // Build final controllers list
            let mut controllers = Vec::new();
            for (controller_mac, device_map) in controller_devices {
                let mut devices = Vec::new();
                for (device_mac, (link_key, le_data)) in &device_map {
                    let device_name = get_device_name_from_cache(&root_key, device_mac, &mut hive)
                        .unwrap_or_else(|| device_mac.clone());

                    let device_type = match (&link_key, &le_data) {
                        (Some(_), Some(_)) => BluetoothDeviceType::DualMode,
                        (Some(_), None) => BluetoothDeviceType::Classic,
                        (None, Some(_)) => BluetoothDeviceType::LowEnergy,
                        (None, None) => continue,
                    };

                    devices.push(BluetoothDevice {
                        name: Some(device_name),
                        address: MacAddress::from_str(device_mac).unwrap_or(MacAddress::default()),
                        device_id: None,
                        device_type,
                        link_key: link_key.clone(),
                        le_data: le_data.clone(),
                    });
                }

                controllers.push(BluetoothController {
                    name: None,
                    address: MacAddress::from_str(&controller_mac).unwrap_or(MacAddress::default()),
                    devices,
                });
            }

            Ok(BluetoothData {
                host: HostDistributions::Windows,
                controllers,
                utc_timestamp: Utc::now(),
                source_path: hive_path_str,
            })
        },
    )
    .await??;

    Ok(bluetooth_data)
}

fn parse_controller_devices(
    controller_key: &KeyNode,
    root_key: &KeyNode,
    hive: &mut Hive<File, CleanHive>,
) -> std::result::Result<Vec<BluetoothDevice>, Box<dyn std::error::Error + Send + Sync>> {
    let mut device_map: HashMap<String, (Option<BluetoothLinkKey>, Option<BluetoothLowEnergyKey>)> = HashMap::new();

    // First pass: collect LE devices (subkeys)
    for device_connected in (*controller_key.subkeys(hive)
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?).iter() {
        let device_key = device_connected.borrow();
        let device_mac = device_key.name().to_string();
        let device_info = device_key.values();

        let get_binary_hex = |name: &str| -> Option<String> {
            device_info
                .iter()
                .find(|x| x.name() == name)
                .and_then(|x| match x.value() {
                    RegistryValue::RegBinary(bytes) => {
                        Some(bytes.iter().map(|b| format!("{:02X}", b)).collect())
                    }
                    _ => None,
                })
        };

        let get_dword = |name: &str| -> Option<u32> {
            device_info
                .iter()
                .find(|x| x.name() == name)
                .and_then(|x| match x.value() {
                    RegistryValue::RegDWord(v) => Some(*v),
                    _ => None,
                })
        };

        let get_qword_string = |name: &str| -> Option<String> {
            device_info
                .iter()
                .find(|x| x.name() == name)
                .and_then(|x| match x.value() {
                    RegistryValue::RegQWord(v) => Some(v.to_string()),
                    _ => None,
                })
        };

        let le_data = parse_le_values(&get_binary_hex, &get_dword, &get_qword_string);

        device_map.insert(device_mac, (None, le_data));
    }

    // Second pass: collect Classic devices (values)
    for device_connected in controller_key.values() {
        if device_connected.name().len() != 12 {
            println!("Skipping extra key: {}", device_connected.name());
            continue;
        }

        let device_mac = device_connected.name().to_string();
        let link_key = match device_connected.value() {
            RegistryValue::RegBinary(bytes) if !bytes.is_empty() => {
                Some(BluetoothLinkKey {
                    key: bytes.iter().map(|b| format!("{:02X}", b)).collect(),
                    key_type: None,
                    pin_length: None,
                })
            }
            _ => None,
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

fn parse_le_values(
    get_binary_hex: &dyn Fn(&str) -> Option<String>,
    get_dword: &dyn Fn(&str) -> Option<u32>,
    get_qword_string: &dyn Fn(&str) -> Option<String>,
) -> Option<BluetoothLowEnergyKey> {
    let ltk_key = get_binary_hex("LTK");
    let long_term_key = ltk_key.map(|key| LongTermKeyData {
        key,
        authenticated: None,
        key_length: get_dword("KeyLength"),
        ediv: get_dword("EDIV"),
        rand: get_qword_string("ERand"),
    });
    let local_csrk = get_binary_hex("CSRK").map(|key| SignatureKeyData {
        key,
        counter: None,
        authenticated: None,
    });
    let remote_csrk = get_binary_hex("CSRKInbound").map(|key| SignatureKeyData {
        key,
        counter: None,
        authenticated: None,
    });
    let irk = get_binary_hex("IRK");

    if long_term_key.is_none() && irk.is_none() {
        return None;
    }

    Some(BluetoothLowEnergyKey {
        long_term_key,
        peripheral_long_term_key: None,
        identity_resolving_key: irk,
        local_signature_key: local_csrk,
        remote_signature_key: remote_csrk,
        address_type: None,
    })
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
        RegistryValue::RegSZ(s) => Some(s.trim().trim_end_matches('\0').to_owned()),
        RegistryValue::RegExpandSZ(s) => Some(s.trim().trim_end_matches('\0').to_owned()),
        RegistryValue::RegBinary(b) => {
            if let Ok(s) = String::from_utf8(b.to_vec()) {
                Some(s.trim().trim_end_matches('\0').to_owned())
            } else {
                None
            }
        },
        RegistryValue::RegMultiSZ(s) => Some(s.join("\n")),
        _ => None,
    }
}
