use std::process::Command as StdCommand;

use bluetooth_model::BluetoothData;

use crate::api::message::Message;
use crate::elevated::{is_elevated, relative_command_path, run_elevated};

#[taurpc::procedures(path = "linux", export_to = "../bindings.ts")]
pub trait LinuxApi {
    async fn parse_local_config() -> Message<BluetoothData>;
}

#[derive(Clone)]
pub struct LinuxApiImpl;

#[taurpc::resolvers]
impl LinuxApi for LinuxApiImpl {
    async fn parse_local_config(self) -> Message<BluetoothData> {
        let path = match relative_command_path("elevated_scrapper") {
            Ok(p) => p,
            Err(e) => return Message::Error(e),
        };

        let output = if is_elevated() {
            StdCommand::new(&path)
                .arg("scan")
                .arg("--privileged")
                .output()
                .map_err(|e| format!("Failed to run elevated scrapper: {}", e))
        } else {
            run_elevated(&path, &["scan", "--privileged"])
        };

        let output = match output {
            Ok(o) => o,
            Err(e) => return Message::Error(e),
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Message::Error(format!(
                "Elevated scrapper exited with {}: {}",
                output.status, stderr
            ));
        }

        match serde_json::from_slice::<BluetoothData>(&output.stdout) {
            Ok(data) => Message::Success(data),
            Err(e) => Message::Error(format!("Failed to parse Bluetooth data: {}", e)),
        }
    }
}
