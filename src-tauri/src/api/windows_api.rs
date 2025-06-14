use std::{fs, path::Path};

use crate::{api::message::Message, bluetooth::bluetooth_data::BluetoothData};


pub struct WindowsHiveData {
	pub path: String,
}

#[taurpc::procedures()]
pub trait WindowsApi {
    async fn parse_windows_hive(path_str: String) -> Message<BluetoothData>;
}

#[derive(Clone)]
pub struct WindowsImpl;

#[taurpc::resolvers]
impl WindowsApi for WindowsImpl {
    async fn parse_windows_hive(self, path_str: String) -> Message<BluetoothData> {
				let path = Path::new(&path_str);

				if (!path.exists()) {

				}


				return Message::Error("File does not exist".to_string());
    }
}
