use bluetooth_model::{BluetoothDevice, HostDistributions};
use serde::{Deserialize, Serialize};

use crate::api::message::Message;

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct SyncProposal {
    pub source_device: BluetoothDevice,
    pub target_device: BluetoothDevice,
    pub source_os: HostDistributions,
    pub target_os: HostDistributions,
    pub action: String,
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
        println!("Received sync request with {} proposals", request.proposals.len());

        let mut applied_count = 0;
        let mut failed_count = 0;
        let mut errors = Vec::new();

        for proposal in &request.proposals {
            match apply_single_proposal(proposal).await {
                Ok(_) => {
                    applied_count += 1;
                    println!("Successfully applied sync for device: {}",
                        proposal.source_device.address);
                }
                Err(e) => {
                    failed_count += 1;
                    let error_msg = format!("Failed to sync device {}: {}",
                        proposal.source_device.address, e);
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
    // TODO: Implement the actual Bluetooth key synchronization logic
    // This would involve:
    // 1. Extracting the Bluetooth keys from the source device
    // 2. Writing them to the target OS's Bluetooth configuration
    // 3. Handling different key formats between Windows and Linux

    println!("Applying sync proposal:");
    println!("  Source: {} ({:?}) -> Target: {} ({:?})",
        proposal.source_device.name.as_deref().unwrap_or("Unknown"),
        proposal.source_os,
        proposal.target_device.name.as_deref().unwrap_or("Unknown"),
        proposal.target_os
    );

    // For now, simulate the operation
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Simulate some failures for demonstration
    if proposal.source_device.link_key.is_none() && proposal.source_device.le_data.is_none() {
        return Err("No pairing keys found on source device".to_string());
    }

    Ok(())
}
