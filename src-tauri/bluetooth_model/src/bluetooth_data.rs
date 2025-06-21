use chrono::{DateTime, Utc};
use mac_address::MacAddress;

#[app_macros::ipc_type]
pub struct BluetoothData {
    pub host: HostDistributions,
    pub controllers: Vec<BluetoothController>,
    pub utc_timestamp: DateTime<Utc>,
    pub source_path: String,
}

impl Default for BluetoothData {
    fn default() -> Self {
        Self {
            host: HostDistributions::Linux,
            controllers: Vec::new(),
            utc_timestamp: Utc::now(),
            source_path: "".to_string(),
        }
    }
}

#[app_macros::ipc_type]
pub struct BluetoothController {
    pub name: Option<String>,
    pub address: MacAddress,
    pub devices: Vec<BluetoothDevice>,
}

#[app_macros::ipc_type]
pub enum HostDistributions {
    Windows,
    Linux,
}

#[app_macros::ipc_type]
pub enum BluetoothDeviceType {
    Classic,
    LowEnergy,
    DualMode,
}

#[app_macros::ipc_type]
pub struct BluetoothDevice {
    pub name: Option<String>,
    pub address: MacAddress,
    pub device_type: BluetoothDeviceType,
    pub link_key: Option<String>,
    pub le_data: Option<BluetoothLowEnergyKey>,
}

#[app_macros::ipc_type]
pub struct BluetoothLowEnergyKey {
    pub identity_resolving_key: Option<String>,
    pub local_signature_key: Option<String>,
    pub long_term_key: Option<String>,
    pub rand: Option<String>,
    pub ediv: Option<String>,
}
