use bluetooth_model::{BluetoothDevice, HostDistributions};
use serde::{Deserialize, Serialize};

use crate::api::message::Message;

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(tag = "action")]
pub enum SyncProposal {
    CopyKeys {
        source_device: BluetoothDevice,
        target_device: BluetoothDevice,
        source_os: HostDistributions,
        target_os: HostDistributions,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct SyncResult {
    pub success: bool,
    pub applied_count: usize,
    pub failed_count: usize,
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
        println!(
            "Received sync request with {} proposals",
            request.proposals.len()
        );

        let mut applied_count = 0;
        let mut failed_count = 0;
        let mut errors = Vec::new();

        for proposal in &request.proposals {
            match apply_single_proposal(proposal).await {
                Ok(_) => {
                    applied_count += 1;
                    match proposal {
                        SyncProposal::CopyKeys {
                            source_device,
                            target_os,
                            ..
                        } => {
                            println!(
                                "Successfully copied keys for device: {} -> {:?}",
                                source_device.address, target_os
                            );
                        }
                        SyncProposal::DeleteDevice { device, os, .. } => {
                            println!(
                                "Successfully deleted device: {} from {:?}",
                                device.address, os
                            );
                        }
                    }
                }
                Err(e) => {
                    failed_count += 1;
                    let device_addr = match proposal {
                        SyncProposal::CopyKeys { source_device, .. } => {
                            &source_device.address
                        }
                        SyncProposal::DeleteDevice { device, .. } => &device.address,
                    };
                    let error_msg = format!("Failed for device {}: {}", device_addr, e);
                    errors.push(error_msg.clone());
                    println!("{}", error_msg);
                }
            }
        }

        let result = SyncResult {
            success: failed_count == 0,
            applied_count,
            failed_count,
            errors,
        };

        Message::Success(result)
    }
}

async fn apply_single_proposal(proposal: &SyncProposal) -> Result<(), String> {
    match proposal {
        SyncProposal::CopyKeys {
            source_device,
            target_device,
            source_os,
            target_os,
        } => {
            // TODO: Implement actual key copy logic
            println!(
                "Copying keys: {} ({:?}) -> {} ({:?})",
                source_device.name.as_deref().unwrap_or("Unknown"),
                source_os,
                target_device.name.as_deref().unwrap_or("Unknown"),
                target_os
            );

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            if source_device.link_key.is_none() && source_device.le_data.is_none() {
                return Err("No pairing keys found on source device".to_string());
            }

            Ok(())
        }
        SyncProposal::DeleteDevice {
            device,
            os,
            controller_address,
        } => {
            // TODO: Implement actual device deletion logic
            // Linux: remove /var/lib/bluetooth/<controller>/<device>/
            // Windows: remove registry keys under BTHPORT\Parameters\Keys\<controller>\<device>
            println!(
                "Deleting device: {} ({:?}) from controller {}",
                device.name.as_deref().unwrap_or("Unknown"),
                os,
                controller_address
            );

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            Ok(())
        }
    }
}
