use std::path::PathBuf;

use bluetooth_model::BluetoothData;
use elevated_command::Command;
use std::process::Command as StdCommand;
use tauri::{utils::platform, AppHandle, Manager, Runtime};

use crate::api::message::Message;

fn relative_command_path(command: &str) -> PathBuf {
    match platform::current_exe().unwrap().parent() {
        #[cfg(windows)]
        Some(exe_dir) => exe_dir.join(command).with_extension("exe"),
        #[cfg(not(windows))]
        Some(exe_dir) => exe_dir.join(command),
        None => unimplemented!(),
    }
}


#[taurpc::procedures(path = "linux")]
pub trait LinuxApi {
    async fn parse_local_config() -> Message<BluetoothData>;
}

#[derive(Clone)]
pub struct LinuxApiImpl;

#[taurpc::resolvers]
impl LinuxApi for LinuxApiImpl {
    async fn parse_local_config(
        self,
    ) -> Message<BluetoothData> {
				let path = relative_command_path("elevated_scrapper");

				let is_elevated = Command::is_elevated();

				let mut cmd = StdCommand::new(path);
				cmd.arg("scan");
				cmd.arg("--privileged");


				println!("Running command: {:?}", cmd);

				let output = if is_elevated {
						cmd.output().unwrap()
				} else {
						let mut elevated_cmd = Command::new(cmd);
						elevated_cmd.name("Bluetooth Key Sync".to_string());
						elevated_cmd.output().unwrap()
				};

				println!("Output: {:?}", output);

        let data = serde_json::from_slice::<BluetoothData>(&output.stdout);

				if let Err(e) = data {
						return Message::Error(format!("Failed to parse Bluetooth data: {}", e));
				}


        Message::Success(data.unwrap())
    }
}
