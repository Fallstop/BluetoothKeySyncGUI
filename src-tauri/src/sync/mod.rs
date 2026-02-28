pub mod linux_writer;
pub mod windows_writer;

use std::sync::Mutex;

use bluetooth_model::HostDistributions;

use crate::api::sync_api::SyncProposal;

static SYNC_LOCK: Mutex<()> = Mutex::new(());

/// Apply all sync proposals, batching Windows operations into a single hive open/commit cycle.
/// Returns (applied_count, failed_count, errors).
pub fn apply_all_proposals(
    proposals: &[SyncProposal],
    hive_path: Option<&str>,
) -> (u32, u32, Vec<String>) {
    let _lock = SYNC_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let mut applied = 0u32;
    let mut failed = 0u32;
    let mut errors = Vec::new();

    // Separate proposals by target OS
    let mut win_proposals = Vec::new();
    let mut lin_proposals = Vec::new();

    for proposal in proposals {
        match proposal {
            SyncProposal::CopyKeys {
                target_os: HostDistributions::Windows,
                ..
            }
            | SyncProposal::DeleteDevice {
                os: HostDistributions::Windows,
                ..
            } => {
                win_proposals.push(proposal);
            }
            _ => {
                lin_proposals.push(proposal);
            }
        }
    }

    // Batch Windows operations: backup hive, open once, apply all, commit once
    if !win_proposals.is_empty() {
        match hive_path {
            Some(path) => {
                match windows_writer::apply_batch(path, &win_proposals) {
                    Ok(results) => {
                        for result in results {
                            match result {
                                Ok(_) => applied += 1,
                                Err(e) => {
                                    failed += 1;
                                    errors.push(e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        // Hive open, backup, or commit failed — all Windows ops failed
                        failed += win_proposals.len() as u32;
                        errors.push(e);
                    }
                }
            }
            None => {
                failed += win_proposals.len() as u32;
                errors.push("No Windows hive path provided".to_string());
            }
        }
    }

    // Linux operations individually
    let mut any_linux_success = false;
    for proposal in &lin_proposals {
        let result = match proposal {
            SyncProposal::CopyKeys { .. } => linux_writer::copy_keys_to_linux(proposal),
            SyncProposal::DeleteDevice { .. } => linux_writer::delete_device_from_linux(proposal),
        };
        match result {
            Ok(_) => {
                applied += 1;
                any_linux_success = true;
            }
            Err(e) => {
                failed += 1;
                errors.push(e);
            }
        }
    }

    // Restart bluetoothd after Linux writes so synced keys take effect
    if any_linux_success {
        if let Err(e) = linux_writer::restart_bluetooth() {
            errors.push(format!("Warning: keys were written but bluetooth service restart failed: {}", e));
        }
    }

    (applied, failed, errors)
}

/// Validate a hex key string (must be even-length hex chars).
pub fn validate_hex_key(key: &str) -> Result<(), String> {
    if key.is_empty() {
        return Err("Key is empty".to_string());
    }
    if key.len() % 2 != 0 {
        return Err(format!("Key has odd length: {}", key.len()));
    }
    if !key.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Key contains non-hex characters".to_string());
    }
    Ok(())
}

/// Validate that a hex key has the expected byte length.
pub fn validate_key_length(key: &str, expected_bytes: usize, key_name: &str) -> Result<(), String> {
    validate_hex_key(key)?;
    let byte_len = key.len() / 2;
    if byte_len != expected_bytes {
        return Err(format!(
            "{} must be {} bytes ({} hex chars), got {} bytes ({} hex chars)",
            key_name,
            expected_bytes,
            expected_bytes * 2,
            byte_len,
            key.len()
        ));
    }
    Ok(())
}

/// Convert a MAC address to colon-separated format: "AA:BB:CC:DD:EE:FF"
pub fn mac_to_colon_format(mac: &str) -> String {
    let flat: String = mac.replace([':', '-'], "").to_uppercase();
    if flat.len() == 12 {
        format!(
            "{}:{}:{}:{}:{}:{}",
            &flat[0..2],
            &flat[2..4],
            &flat[4..6],
            &flat[6..8],
            &flat[8..10],
            &flat[10..12]
        )
    } else {
        mac.to_uppercase()
    }
}

/// Convert a MAC address to flat format: "AABBCCDDEEFF"
pub fn mac_to_flat_format(mac: &str) -> String {
    mac.replace([':', '-'], "").to_uppercase()
}

/// Decode a hex string to bytes.
pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    validate_hex_key(hex)?;
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| format!("Invalid hex at position {}: {}", i, e))
        })
        .collect()
}
