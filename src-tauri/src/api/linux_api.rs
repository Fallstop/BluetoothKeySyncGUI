use bluetooth_model::BluetoothData;
use tauri::{AppHandle, Manager, Runtime};

use crate::api::message::Message;

#[taurpc::procedures(path = "linux")]
pub trait LinuxApi {
    async fn parse_local_config<R: Runtime>(app_handle: AppHandle<R>) -> Message<BluetoothData>;
}

#[derive(Clone)]
pub struct LinuxApiImpl;

#[taurpc::resolvers]
impl LinuxApi for LinuxApiImpl {
    async fn parse_local_config<R: Runtime>(
        self,
        app_handle: AppHandle<R>,
    ) -> Message<BluetoothData> {
        // let asset_thing =  app_handle.shell().;

        let data = BluetoothData::default();

        Message::Error("Linux API not implemented yet".to_string())
    }
}
