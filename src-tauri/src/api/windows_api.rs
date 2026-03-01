use std::path::Path;

use bluetooth_model::BluetoothData;

use crate::{api::message::Message, bluetooth::hive_parse};


#[taurpc::procedures(path = "windows")]
pub trait WindowsApi {
    async fn parse_windows_hive(path_str: String) -> Message<BluetoothData>;
}

#[derive(Clone)]
pub struct WindowsImpl;

#[taurpc::resolvers]
impl WindowsApi for WindowsImpl {
    async fn parse_windows_hive(self, path_str: String) -> Message<BluetoothData> {
        let path = Path::new(&path_str);

        if !path.exists() {
            return Message::Error(format!("Path does not exist: {}", path_str));
        }
        if !path.is_absolute() {
            return Message::Error(format!("Path is not absolute: {}", path_str));
        }

        let final_path = if path.is_dir() {
            // Automatically append the default hive file location if a root drive directory is provided
            // on Windows 10 the root folder is "Windows", on Windows 7 it is "WINDOWS"

            let possible_hive_paths = [
                "Windows/System32/config/SYSTEM",
                "WINDOWS/System32/config/SYSTEM",
            ];

            let mut found_path = None;
            for possible_path in possible_hive_paths.iter() {
                let full_path = path.join(possible_path);
                if full_path.exists() && full_path.is_file() {
                    found_path = Some(full_path);
                    break;
                }
            }

            if let Some(found) = found_path {
                found
            } else {
                return Message::Error(format!("No hive file found in directory: {}", path_str));
            }
        } else {
            path.to_path_buf()
        };

        // Now parse
        match hive_parse::extract_hive_data(&final_path).await {
            Ok(data) => Message::Success(data),
            Err(error) => Message::Error(error.to_string()),
        }
    }
}
