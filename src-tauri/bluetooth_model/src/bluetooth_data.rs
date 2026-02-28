use chrono::{DateTime, Utc};
use mac_address::MacAddress;

/// Deserializes an `Option<u32>` from either a JSON number or a JSON string.
mod string_or_u32 {
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrU32 {
            U32(u32),
            Str(String),
        }

        match Option::<StringOrU32>::deserialize(deserializer)? {
            Some(StringOrU32::U32(v)) => Ok(Some(v)),
            Some(StringOrU32::Str(s)) => Ok(s.parse().ok()),
            None => Ok(None),
        }
    }
}

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
    pub key_type: Option<u8>,
    pub pin_length: Option<u8>,
}

#[app_macros::ipc_type]
pub struct LongTermKeyData {
    pub key: String,
    pub authenticated: Option<bool>,
    pub key_length: Option<u32>,
    #[serde(default, deserialize_with = "string_or_u32::deserialize")]
    pub ediv: Option<u32>,
    pub rand: Option<String>,
}

#[app_macros::ipc_type]
pub struct SignatureKeyData {
    pub key: String,
    pub counter: Option<u32>,
    pub authenticated: Option<bool>,
}

#[app_macros::ipc_type]
pub struct BluetoothLowEnergyKey {
    pub long_term_key: Option<LongTermKeyData>,
    pub peripheral_long_term_key: Option<LongTermKeyData>,
    pub identity_resolving_key: Option<String>,
    pub local_signature_key: Option<SignatureKeyData>,
    pub remote_signature_key: Option<SignatureKeyData>,
    pub address_type: Option<String>,
}
