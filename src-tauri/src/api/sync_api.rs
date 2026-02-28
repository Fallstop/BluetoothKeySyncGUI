use bluetooth_model::{BluetoothDevice, HostDistributions};
use serde::{Deserialize, Serialize};

use crate::api::message::Message;
use crate::sync;

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(tag = "action")]
pub enum SyncProposal {
    CopyKeys {
        source_device: BluetoothDevice,
        target_device: BluetoothDevice,
        source_os: HostDistributions,
        target_os: HostDistributions,
        source_controller_address: String,
        target_controller_address: String,
    },
    DeleteDevice {
        device: BluetoothDevice,
        os: HostDistributions,
        controller_address: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct SyncRequest {
    pub proposals: Vec<SyncProposal>,
    pub windows_hive_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct SyncResult {
    pub success: bool,
    pub applied_count: u32,
    pub failed_count: u32,
    pub errors: Vec<String>,
}

#[taurpc::procedures(path = "sync")]
pub trait SyncApi {
    async fn apply_sync_proposals(request: SyncRequest) -> Message<SyncResult>;
}

#[derive(Clone)]
pub struct SyncApiImpl;

#[taurpc::resolvers]
impl SyncApi for SyncApiImpl {
    async fn apply_sync_proposals(self, request: SyncRequest) -> Message<SyncResult> {
        let result = tokio::task::spawn_blocking(move || {
            sync::apply_all_proposals(
                &request.proposals,
                request.windows_hive_path.as_deref(),
            )
        })
        .await;

        match result {
            Ok((applied_count, failed_count, errors)) => {
                if applied_count == 0 && failed_count > 0 {
                    Message::Error(format!(
                        "All {} operations failed: {}",
                        failed_count,
                        errors.join("; ")
                    ))
                } else {
                    Message::Success(SyncResult {
                        success: failed_count == 0,
                        applied_count,
                        failed_count,
                        errors,
                    })
                }
            }
            Err(e) => Message::Error(format!("Sync task failed: {}", e)),
        }
    }
}
