use std::process::Command as StdCommand;

use bluetooth_model::batch::{BatchOperation, BatchRequest, BatchResponse};
use bluetooth_model::BluetoothData;

use crate::api::sync_api::SyncProposal;
use crate::elevated::{is_elevated, relative_command_path, run_elevated};
use crate::sync::mac_to_colon_format;

use base64::Engine;

pub fn copy_keys_to_linux(proposal: &SyncProposal) -> Result<(), String> {
    let (source_device, target_controller_address) = match proposal {
        SyncProposal::CopyKeys {
            source_device,
            target_controller_address,
            ..
        } => (source_device, target_controller_address),
        _ => return Err("Expected CopyKeys proposal".to_string()),
    };

    let controller_addr = mac_to_colon_format(target_controller_address);
    let device_addr = mac_to_colon_format(&source_device.address.to_string());

    // Serialize the source device to JSON, then base64-encode it
    let json = serde_json::to_string(source_device)
        .map_err(|e| format!("Failed to serialize device: {}", e))?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&json);

    let scrapper_path = relative_command_path("elevated_scrapper")?;
    let args = [
        "write-keys",
        "--privileged",
        "--controller",
        &controller_addr,
        "--device",
        &device_addr,
        "--data",
        &b64,
    ];

    let output = if is_elevated() {
        StdCommand::new(&scrapper_path)
            .args(&args)
            .output()
            .map_err(|e| format!("Failed to run elevated scrapper: {}", e))
    } else {
        run_elevated(&scrapper_path, &args)
    }?;

    parse_scrapper_result(&output)
}

pub fn delete_device_from_linux(proposal: &SyncProposal) -> Result<(), String> {
    let (controller_address, device) = match proposal {
        SyncProposal::DeleteDevice {
            controller_address,
            device,
            ..
        } => (controller_address, device),
        _ => return Err("Expected DeleteDevice proposal".to_string()),
    };

    let controller_addr = mac_to_colon_format(controller_address);
    let device_addr = mac_to_colon_format(&device.address.to_string());

    let scrapper_path = relative_command_path("elevated_scrapper")?;
    let args = [
        "delete-device",
        "--privileged",
        "--controller",
        &controller_addr,
        "--device",
        &device_addr,
    ];

    let output = if is_elevated() {
        StdCommand::new(&scrapper_path)
            .args(&args)
            .output()
            .map_err(|e| format!("Failed to run elevated scrapper: {}", e))
    } else {
        run_elevated(&scrapper_path, &args)
    }?;

    parse_scrapper_result(&output)
}

pub fn restart_bluetooth() -> Result<(), String> {
    let scrapper_path = relative_command_path("elevated_scrapper")?;
    let args = ["restart-bluetooth", "--privileged"];

    let output = if is_elevated() {
        StdCommand::new(&scrapper_path)
            .args(&args)
            .output()
            .map_err(|e| format!("Failed to run elevated scrapper: {}", e))
    } else {
        run_elevated(&scrapper_path, &args)
    }?;

    parse_scrapper_result(&output)
}

/// Result of a batched Linux operation set.
pub struct LinuxBatchResult {
    pub applied: u32,
    pub failed: u32,
    pub errors: Vec<String>,
    pub refreshed_linux: Option<BluetoothData>,
}

/// Execute all Linux proposals + restart + rescan in a single elevated invocation.
pub fn batch_linux_operations(proposals: &[&SyncProposal]) -> Result<LinuxBatchResult, String> {
    let mut operations = Vec::new();
    // Track which operation indices correspond to proposals (for result mapping)
    let mut proposal_indices = Vec::new();

    for proposal in proposals {
        let op = match proposal {
            SyncProposal::CopyKeys {
                source_device,
                target_controller_address,
                ..
            } => {
                let controller_addr = mac_to_colon_format(target_controller_address);
                let device_addr = mac_to_colon_format(&source_device.address.to_string());
                let json = serde_json::to_string(source_device)
                    .map_err(|e| format!("Failed to serialize device: {}", e))?;
                let b64 = base64::engine::general_purpose::STANDARD.encode(&json);
                BatchOperation::WriteKeys {
                    controller: controller_addr,
                    device: device_addr,
                    data: b64,
                }
            }
            SyncProposal::DeleteDevice {
                controller_address,
                device,
                ..
            } => {
                let controller_addr = mac_to_colon_format(controller_address);
                let device_addr = mac_to_colon_format(&device.address.to_string());
                BatchOperation::DeleteDevice {
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
    operations.push(BatchOperation::RestartBluetooth);
    operations.push(BatchOperation::Scan);

    let request = BatchRequest { operations };
    let json = serde_json::to_string(&request)
        .map_err(|e| format!("Failed to serialize BatchRequest: {}", e))?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&json);

    let scrapper_path = relative_command_path("elevated_scrapper")?;
    let args = ["batch", "--privileged", "--data", &b64];

    let output = if is_elevated() {
        StdCommand::new(&scrapper_path)
            .args(&args)
            .output()
            .map_err(|e| format!("Failed to run elevated scrapper: {}", e))
    } else {
        run_elevated(&scrapper_path, &args)
    }?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Elevated scrapper batch exited with {}: {}",
            output.status, stderr
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let response: BatchResponse = serde_json::from_str(&stdout).map_err(|e| {
        format!(
            "Failed to parse BatchResponse: {}. stdout: {}",
            e, stdout
        )
    })?;

    // Map results back to proposals
    let mut applied = 0u32;
    let mut failed = 0u32;
    let mut errors = Vec::new();

    for (i, &op_index) in proposal_indices.iter().enumerate() {
        if let Some(result) = response.results.iter().find(|r| r.index == op_index) {
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
    if let Some(result) = response.results.iter().find(|r| r.index == restart_index) {
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
        refreshed_linux: response.scan_data,
    })
}

fn parse_scrapper_result(output: &std::process::Output) -> Result<(), String> {
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Elevated scrapper exited with {}: {}",
            output.status, stderr
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Try to parse JSON result from stdout
    #[derive(serde::Deserialize)]
    struct ScrapperResult {
        success: bool,
        #[serde(default)]
        error: Option<String>,
    }

    match serde_json::from_str::<ScrapperResult>(&stdout) {
        Ok(result) => {
            if result.success {
                Ok(())
            } else {
                Err(result
                    .error
                    .unwrap_or_else(|| "Unknown error from elevated scrapper".to_string()))
            }
        }
        Err(e) => {
            // Process succeeded but output is not valid JSON — this indicates a bug
            Err(format!(
                "Could not parse scrapper JSON output: {}. stdout: {}",
                e, stdout
            ))
        }
    }
}
