use bluetooth_model::worker_protocol::{WorkerOperation, WorkerResponseData, WorkerResult};
use bluetooth_model::BluetoothData;

use crate::api::message::Message;
use crate::elevated_worker::get_worker;

#[taurpc::procedures(path = "linux", export_to = "../bindings.ts")]
pub trait LinuxApi {
    async fn parse_local_config() -> Message<BluetoothData>;
}

#[derive(Clone)]
pub struct LinuxApiImpl;

#[taurpc::resolvers]
impl LinuxApi for LinuxApiImpl {
    async fn parse_local_config(self) -> Message<BluetoothData> {
        let worker = get_worker();
        let resp = match worker.send_command(WorkerOperation::Scan).await {
            Ok(r) => r,
            Err(e) => return Message::Error(e),
        };

        match resp.result {
            WorkerResult::Ok { data } => match data {
                Some(WorkerResponseData::ScanResult { bluetooth_data }) => {
                    Message::Success(bluetooth_data)
                }
                _ => Message::Error("Worker returned unexpected response for Scan".to_string()),
            },
            WorkerResult::Err { message } => Message::Error(message),
        }
    }
}
