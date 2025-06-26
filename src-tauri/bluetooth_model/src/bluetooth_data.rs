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
    Corrupted
}

#[app_macros::ipc_type]
pub struct BluetoothDevice {
    pub name: Option<String>,
    pub address: MacAddress,
    pub device_type: BluetoothDeviceType,
    pub device_id: Option<DeviceID>,
    pub link_key: Option<BluetoothLinkKey>,
    pub le_data: Option<BluetoothLowEnergyKey>,
}

#[app_macros::ipc_type]
pub struct DeviceID {
    pub source: Option<u32>,
    pub vendor: Option<u32>,
    pub product: Option<u32>,
    pub version: Option<u32>,
}

#[app_macros::ipc_type]
pub struct BluetoothLinkKey {
    pub key: String,
}

#[app_macros::ipc_type]
pub struct BluetoothLowEnergyKey {
    pub identity_resolving_key: Option<String>,
    pub local_signature_key: Option<String>,
    pub long_term_key: Option<String>,
    pub key_length: Option<u32>,
    pub ediv: Option<String>,
    pub rand: Option<String>,
}

impl BluetoothLowEnergyKey {
    pub fn rank_validity(&self) -> u8 {
        let mut rank = 0;
        if self.long_term_key.is_some() {
            rank += 4;
        }
        if self.identity_resolving_key.is_some() {
            rank += 2;
        }
        if self.local_signature_key.is_some() {
            rank += 1;
        }
        if self.key_length.is_some() {
            rank += 1;
        }
        if self.ediv.is_some() {
            rank += 1;
        }
        if self.rand.is_some() {
            rank += 1;
        }
        rank
    }
}
