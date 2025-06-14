use chrono::{DateTime, Utc};
use mac_address::MacAddress;


#[app_macros::ipc_type]
pub struct BluetoothData {
		pub host: HostDistributions,
		pub controllers: Vec<BluetoothController>,
		pub utc_timestamp: DateTime<Utc>
}

impl Default for BluetoothData {
    fn default() -> Self {
        Self {
            host: HostDistributions::Linux,
            controllers: Vec::new(),
            utc_timestamp: Utc::now(),
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
}

#[app_macros::ipc_type]
pub struct BluetoothDevice {
	pub name: Option<String>,
	pub address: [u8; 6],
	pub device_type: BluetoothDeviceType,
	pub link_key: Option<String>,
}
