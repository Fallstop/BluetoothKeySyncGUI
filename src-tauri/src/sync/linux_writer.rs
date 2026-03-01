use bluetooth_model::worker_protocol::{WorkerOperation, WorkerResponseData, WorkerResult};
use bluetooth_model::BluetoothData;

use crate::api::sync_api::SyncProposal;
use crate::elevated_worker::get_worker;
use crate::sync::mac_to_colon_format;

/// Result of a batched Linux operation set.
pub struct LinuxBatchResult {
    pub applied: u32,
    pub failed: u32,
    pub errors: Vec<String>,
    pub refreshed_linux: Option<BluetoothData>,
}

/// Execute all Linux proposals + restart + rescan via the persistent elevated worker.
pub async fn batch_linux_operations(
    proposals: &[&SyncProposal],
) -> Result<LinuxBatchResult, String> {
    let mut operations = Vec::new();
    let mut proposal_indices = Vec::new();

    for proposal in proposals {
        let op = match proposal {
            SyncProposal::CopyKeys {
                source_device,
                target_device,
                target_controller_address,
                ..
            } => {
                let controller_addr = mac_to_colon_format(target_controller_address);
                let device_addr = mac_to_colon_format(&target_device.address.to_string());
                let json = serde_json::to_string(source_device)
                    .map_err(|e| format!("Failed to serialize device: {}", e))?;
                WorkerOperation::WriteKeys {
                    controller: controller_addr,
                    device: device_addr,
                    data: json,
                }
            }
            SyncProposal::DeleteDevice {
                controller_address,
                device,
                ..
            } => {
                let controller_addr = mac_to_colon_format(controller_address);
                let device_addr = mac_to_colon_format(&device.address.to_string());
                WorkerOperation::DeleteDevice {
                    controller: controller_addr,
                    device: device_addr,
                }
            }
        };
        proposal_indices.push(operations.len());
        operations.push(op);
    }

    // Add restart and scan after all write/delete operations
    let restart_index = operations.len();
    operations.push(WorkerOperation::RestartBluetooth);
    operations.push(WorkerOperation::Scan);

    let worker = get_worker();
    let resp = worker
        .send_command(WorkerOperation::Batch { operations })
        .await
        .map_err(|e| format!("Worker batch command failed: {}", e))?;

    match resp.result {
        WorkerResult::Ok { data } => match data {
            Some(WorkerResponseData::BatchResult {
                results,
                scan_data,
            }) => {
                let mut applied = 0u32;
                let mut failed = 0u32;
                let mut errors = Vec::new();

                for (i, &op_index) in proposal_indices.iter().enumerate() {
                    if let Some(result) = results.iter().find(|r| r.index == op_index) {
                        if result.success {
                            applied += 1;
                        } else {
                            failed += 1;
                            if let Some(ref err) = result.error {
                                let device_desc = match proposals[i] {
                                    SyncProposal::CopyKeys {
                                        source_device, ..
                                    } => source_device
                                        .name
                                        .clone()
                                        .unwrap_or_else(|| source_device.address.to_string()),
                                    SyncProposal::DeleteDevice { device, .. } => device
                                        .name
                                        .clone()
                                        .unwrap_or_else(|| device.address.to_string()),
                                };
                                errors.push(format!("Linux {}: {}", device_desc, err));
                            }
                        }
                    }
                }

                // Check restart result
                if let Some(result) = results.iter().find(|r| r.index == restart_index) {
                    if !result.success {
                        if let Some(ref err) = result.error {
                            errors.push(format!(
                                "Warning: keys were written but bluetooth restart failed: {}",
                                err
                            ));
                        }
                    }
                }

                Ok(LinuxBatchResult {
                    applied,
                    failed,
                    errors,
                    refreshed_linux: scan_data,
                })
            }
            _ => Err("Worker returned unexpected response for Batch".to_string()),
        },
        WorkerResult::Err { message } => Err(message),
    }
}
