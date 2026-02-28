use crate::api::sync_api::SyncProposal;
use crate::sync::{hex_to_bytes, mac_to_flat_format, validate_key_length};

use bluetooth_model::BluetoothDevice;
use hivex::node::NodeHandle;
use hivex::value::Value;
use hivex::{CommitFlags, Hive, OpenFlags, SetValueFlags};

/// Detect the active ControlSet from the hive's Select key.
/// Returns e.g. "ControlSet001", "ControlSet002", etc.
/// Falls back to "ControlSet001" if detection fails.
fn detect_active_controlset(hive: &Hive) -> String {
    let root = match hive.root() {
        Ok(r) => r,
        Err(_) => return "ControlSet001".to_string(),
    };
    let select_node = match hive.node(root).get_child("Select") {
        Some(n) => n,
        None => return "ControlSet001".to_string(),
    };
    let current_handle = match hive.node(select_node).get_value("Current") {
        Ok(v) => v,
        Err(_) => return "ControlSet001".to_string(),
    };
    let n = hive.value(current_handle).downcast_dword();
    if n >= 1 && n <= 9 {
        format!("ControlSet{:03}", n)
    } else {
        "ControlSet001".to_string()
    }
}

/// Apply a batch of Windows-targeted proposals: backup hive, open once, apply all, commit once.
/// Returns Err if backup/open/commit fails (all proposals considered failed).
/// Returns Ok(Vec<Result>) with per-proposal results on successful open+commit.
pub fn apply_batch(
    hive_path: &str,
    proposals: &[&SyncProposal],
) -> Result<Vec<Result<(), String>>, String> {
    // Backup hive before any modifications
    let backup_path = format!("{}.bak", hive_path);
    std::fs::copy(hive_path, &backup_path)
        .map_err(|e| format!("Failed to backup hive to {}: {}", backup_path, e))?;

    let hive = Hive::open(hive_path, OpenFlags::WRITE)
        .map_err(|e| format!("Failed to open hive file: {}", e))?;

    let controlset = detect_active_controlset(&hive);

    let mut results = Vec::new();
    let mut any_success = false;

    for proposal in proposals {
        let result = match proposal {
            SyncProposal::CopyKeys { .. } => apply_copy_keys(&hive, &controlset, proposal),
            SyncProposal::DeleteDevice { .. } => apply_delete_device(&hive, &controlset, proposal),
        };
        if result.is_ok() {
            any_success = true;
        }
        results.push(result);
    }

    // Only commit if any operations succeeded
    if any_success {
        hive.commit(None, CommitFlags::empty())
            .map_err(|e| format!("Failed to commit hive changes: {}", e))?;
    }

    Ok(results)
}

fn apply_copy_keys(hive: &Hive, controlset: &str, proposal: &SyncProposal) -> Result<(), String> {
    let (source_device, target_controller_address) = match proposal {
        SyncProposal::CopyKeys {
            source_device,
            target_controller_address,
            ..
        } => (source_device, target_controller_address),
        _ => return Err("Expected CopyKeys proposal".to_string()),
    };

    let controller_mac = mac_to_flat_format(target_controller_address);
    let device_mac = mac_to_flat_format(&source_device.address.to_string());

    // Write Classic link key to BTHPORT
    if let Some(ref link_key) = source_device.link_key {
        validate_key_length(&link_key.key, 16, "Link key")?;
        write_classic_key(hive, controlset, &controller_mac, &device_mac, link_key)?;
    }

    // Write LE keys to BTHLE
    if let Some(ref le_data) = source_device.le_data {
        write_le_keys(
            hive,
            controlset,
            &controller_mac,
            &device_mac,
            source_device,
            le_data,
        )?;
    }

    Ok(())
}

fn apply_delete_device(hive: &Hive, controlset: &str, proposal: &SyncProposal) -> Result<(), String> {
    let (controller_address, device) = match proposal {
        SyncProposal::DeleteDevice {
            controller_address,
            device,
            ..
        } => (controller_address, device),
        _ => return Err("Expected DeleteDevice proposal".to_string()),
    };

    let controller_mac = mac_to_flat_format(controller_address);
    let device_mac = mac_to_flat_format(&device.address.to_string());

    // Try to delete from BTHPORT (classic key is a value on the controller node; LE is a subkey)
    let bthport_path = format!(
        "{}\\Services\\BTHPORT\\Parameters\\Keys\\{}",
        controlset, controller_mac
    );
    if let Some(bthport_ctrl) = navigate_to_key(hive, &bthport_path)? {
        let bthport_node = hive.node(bthport_ctrl);

        // Try deleting the LE subkey under BTHPORT controller
        if let Some(child_handle) = bthport_node.get_child(&device_mac) {
            let child_node = hive.node(child_handle);
            child_node
                .delete()
                .map_err(|e| format!("Failed to delete BTHPORT LE subkey: {}", e))?;
        }

        // For classic keys, we can't directly delete a single value with hivex's current API,
        // but we can overwrite it with an empty binary value as a soft-delete
        if let Ok(_) = bthport_node.get_value(&*device_mac) {
            bthport_node
                .set_value(
                    SetValueFlags::empty(),
                    &*device_mac,
                    Value::<&str>::Binary((&[] as &[u8]).into()),
                )
                .map_err(|e| format!("Failed to clear BTHPORT classic key: {}", e))?;
        }
    }

    // Try to delete from BTHLE
    let bthle_path = format!(
        "{}\\Services\\BTHLE\\Parameters\\Keys\\{}",
        controlset, controller_mac
    );
    if let Some(bthle_ctrl) = navigate_to_key(hive, &bthle_path)? {
        let bthle_node = hive.node(bthle_ctrl);
        if let Some(device_handle) = bthle_node.get_child(&device_mac) {
            let device_node = hive.node(device_handle);
            device_node
                .delete()
                .map_err(|e| format!("Failed to delete BTHLE device subkey: {}", e))?;
        }
    }

    Ok(())
}

fn write_classic_key(
    hive: &Hive,
    controlset: &str,
    controller_mac: &str,
    device_mac: &str,
    link_key: &bluetooth_model::BluetoothLinkKey,
) -> Result<(), String> {
    let key_bytes =
        hex_to_bytes(&link_key.key).map_err(|e| format!("Invalid link key hex: {}", e))?;

    let path = format!(
        "{}\\Services\\BTHPORT\\Parameters\\Keys\\{}",
        controlset, controller_mac
    );

    let node = navigate_or_create_key(hive, &path)?;
    let selected = hive.node(node);

    selected
        .set_value(
            SetValueFlags::empty(),
            device_mac,
            Value::<&str>::Binary(key_bytes.as_slice().into()),
        )
        .map_err(|e| format!("Failed to set link key value: {}", e))?;

    Ok(())
}

fn write_le_keys(
    hive: &Hive,
    controlset: &str,
    controller_mac: &str,
    device_mac: &str,
    _source_device: &BluetoothDevice,
    le_data: &bluetooth_model::BluetoothLowEnergyKey,
) -> Result<(), String> {
    let path = format!(
        "{}\\Services\\BTHLE\\Parameters\\Keys\\{}\\{}",
        controlset, controller_mac, device_mac
    );

    let node = navigate_or_create_key(hive, &path)?;
    let selected = hive.node(node);

    // LTK: Use long_term_key if present, otherwise fall back to peripheral_long_term_key
    let effective_ltk = le_data
        .long_term_key
        .as_ref()
        .or(le_data.peripheral_long_term_key.as_ref());

    if let Some(ltk) = effective_ltk {
        validate_key_length(&ltk.key, 16, "LTK")?;
        let ltk_bytes =
            hex_to_bytes(&ltk.key).map_err(|e| format!("Invalid LTK hex: {}", e))?;
        selected
            .set_value(
                SetValueFlags::empty(),
                "LTK",
                Value::<&str>::Binary(ltk_bytes.as_slice().into()),
            )
            .map_err(|e| format!("Failed to set LTK: {}", e))?;

        // Write Authenticated DWORD (default to 0 = unauthenticated if not set)
        let auth_val = if ltk.authenticated.unwrap_or(false) { 1u32 } else { 0u32 };
        selected
            .set_value(
                SetValueFlags::empty(),
                "Authenticated",
                Value::<&str>::Dword(auth_val),
            )
            .map_err(|e| format!("Failed to set Authenticated: {}", e))?;

        if let Some(key_length) = ltk.key_length {
            selected
                .set_value(
                    SetValueFlags::empty(),
                    "KeyLength",
                    Value::<&str>::Dword(key_length),
                )
                .map_err(|e| format!("Failed to set KeyLength: {}", e))?;
        }

        if let Some(ediv) = ltk.ediv {
            selected
                .set_value(
                    SetValueFlags::empty(),
                    "EDIV",
                    Value::<&str>::Dword(ediv),
                )
                .map_err(|e| format!("Failed to set EDIV: {}", e))?;
        }

        if let Some(ref rand_str) = ltk.rand {
            let rand_val: u64 = rand_str
                .parse()
                .map_err(|e| format!("Invalid ERand value '{}': {}", rand_str, e))?;
            selected
                .set_value(
                    SetValueFlags::empty(),
                    "ERand",
                    Value::<&str>::Qword(rand_val),
                )
                .map_err(|e| format!("Failed to set ERand: {}", e))?;
        }
    }

    // IRK
    if let Some(ref irk) = le_data.identity_resolving_key {
        validate_key_length(irk, 16, "IRK")?;
        let irk_bytes = hex_to_bytes(irk).map_err(|e| format!("Invalid IRK hex: {}", e))?;
        selected
            .set_value(
                SetValueFlags::empty(),
                "IRK",
                Value::<&str>::Binary(irk_bytes.as_slice().into()),
            )
            .map_err(|e| format!("Failed to set IRK: {}", e))?;
    }

    // CSRK (local signature key)
    if let Some(ref local_csrk) = le_data.local_signature_key {
        validate_key_length(&local_csrk.key, 16, "CSRK")?;
        let csrk_bytes =
            hex_to_bytes(&local_csrk.key).map_err(|e| format!("Invalid CSRK hex: {}", e))?;
        selected
            .set_value(
                SetValueFlags::empty(),
                "CSRK",
                Value::<&str>::Binary(csrk_bytes.as_slice().into()),
            )
            .map_err(|e| format!("Failed to set CSRK: {}", e))?;
    }

    // CSRKInbound (remote signature key)
    if let Some(ref remote_csrk) = le_data.remote_signature_key {
        validate_key_length(&remote_csrk.key, 16, "CSRKInbound")?;
        let csrk_bytes = hex_to_bytes(&remote_csrk.key)
            .map_err(|e| format!("Invalid CSRKInbound hex: {}", e))?;
        selected
            .set_value(
                SetValueFlags::empty(),
                "CSRKInbound",
                Value::<&str>::Binary(csrk_bytes.as_slice().into()),
            )
            .map_err(|e| format!("Failed to set CSRKInbound: {}", e))?;
    }

    Ok(())
}

/// Navigate to a registry key path, returning the node handle if it exists.
fn navigate_to_key(hive: &Hive, path: &str) -> Result<Option<NodeHandle>, String> {
    let root = hive
        .root()
        .map_err(|e| format!("Failed to get hive root: {}", e))?;

    let mut current = root;
    for component in path.split('\\') {
        let node = hive.node(current);
        match node.get_child(component) {
            Some(child) => current = child,
            None => return Ok(None),
        }
    }

    Ok(Some(current))
}

/// Navigate to a registry key path, creating intermediate nodes as needed.
fn navigate_or_create_key(hive: &Hive, path: &str) -> Result<NodeHandle, String> {
    let root = hive
        .root()
        .map_err(|e| format!("Failed to get hive root: {}", e))?;

    let mut current = root;
    for component in path.split('\\') {
        let node = hive.node(current);
        match node.get_child(component) {
            Some(child) => current = child,
            None => {
                current = node
                    .node_add_child(component)
                    .map_err(|e| format!("Failed to create key '{}': {}", component, e))?;
            }
        }
    }

    Ok(current)
}
