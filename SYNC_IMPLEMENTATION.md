# Sync System — Implementation Reference

Quick reference for the sync writers (Windows and Linux).

## Architecture

```
sync_api.rs (RPC entry) → sync/mod.rs (dispatcher) → sync/windows_writer.rs
                                                    → sync/linux_writer.rs → elevated_scrapper (root)
```

## Data Flow

1. Frontend builds `SyncRequest` with `proposals[]` + `windows_hive_path`
2. `sync_api.rs::apply_sync_proposals()` iterates proposals, calls `sync::apply_proposal()`
3. Dispatcher routes by `target_os` (CopyKeys) or `os` (DeleteDevice)

## SyncProposal Variants

```rust
CopyKeys {
    source_device: BluetoothDevice,    // Device with the keys to copy FROM
    target_device: BluetoothDevice,    // Device to copy keys TO
    source_os: HostDistributions,
    target_os: HostDistributions,      // Determines which writer is called
    source_controller_address: String, // MAC of controller on source OS
    target_controller_address: String, // MAC of controller on target OS
}

DeleteDevice {
    device: BluetoothDevice,
    os: HostDistributions,             // Determines which writer is called
    controller_address: String,
}
```

## Windows Writer (`sync/windows_writer.rs`)

**Library**: `hivex` crate (wraps libhivex C library). Requires `libhivex-dev` system package.

**Key API pattern** (select-then-operate):
```rust
let hive = Hive::open(path, OpenFlags::WRITE)?;
let node = hive.node(handle);
node.set_value(SetValueFlags::empty(), "key_name", Value::<&str>::Binary(bytes.as_slice().into()))?;
node.set_value(SetValueFlags::empty(), "key_name", Value::<&str>::Dword(42))?;
node.set_value(SetValueFlags::empty(), "key_name", Value::<&str>::Qword(123u64))?;
hive.commit(None, CommitFlags::empty())?;
```

**Registry paths**:
- Classic keys: `ControlSet001\Services\BTHPORT\Parameters\Keys\{controller}\{device}` (REG_BINARY value)
- LE keys: `ControlSet001\Services\BTHLE\Parameters\Keys\{controller}\{device}\` (subkey with LTK, IRK, CSRK, etc.)

**MAC format**: Flat uppercase (`AABBCCDDEEFF`) in registry keys.

**LE cross-role**: Windows stores a single LTK. Linux distinguishes `[LongTermKey]` vs `[PeripheralLongTermKey]`. When writing to Windows, use whichever is available (prefer `long_term_key`, fall back to `peripheral_long_term_key`).

## Linux Writer (`sync/linux_writer.rs`)

**Approach**: Delegates to the elevated scrapper binary via `run_elevated()` (shared `elevated.rs`).

**Elevated scrapper subcommands**:
- `write-keys --controller AA:BB:CC:DD:EE:FF --device AA:BB:CC:DD:EE:FF --data <base64_json>`
- `delete-device --controller AA:BB:CC:DD:EE:FF --device AA:BB:CC:DD:EE:FF`

**Key data is passed as base64-encoded JSON** of `BluetoothDevice` (can't use stdin since sudo uses it for password).

**File paths**: `/var/lib/bluetooth/{controller_mac}/{device_mac}/info` (INI format, parsed by `DeviceInfo`).

**DeviceInfo already has setters**: `set_link_key()`, `set_le_pairing_data()`, `set_name()` — the elevated scrapper just deserializes the incoming device and calls these.

**MAC format**: Colon-separated (`AA:BB:CC:DD:EE:FF`) in Linux filesystem paths.

## BluetoothDevice Fields (bluetooth_model)

```rust
BluetoothDevice {
    name: Option<String>,
    address: MacAddress,
    device_type: BluetoothDeviceType,  // Classic | LowEnergy | DualMode | Corrupted
    device_id: Option<DeviceID>,
    link_key: Option<BluetoothLinkKey>,
    le_data: Option<BluetoothLowEnergyKey>,
}

BluetoothLinkKey { key: String, key_type: Option<u8>, pin_length: Option<u8> }

BluetoothLowEnergyKey {
    long_term_key: Option<LongTermKeyData>,
    peripheral_long_term_key: Option<LongTermKeyData>,
    identity_resolving_key: Option<String>,
    local_signature_key: Option<SignatureKeyData>,
    remote_signature_key: Option<SignatureKeyData>,
    address_type: Option<String>,
}

LongTermKeyData { key: String, authenticated: Option<u8>, key_length: Option<u32>, ediv: Option<u32>, rand: Option<String> }
SignatureKeyData { key: String, counter: Option<u32>, authenticated: Option<u8> }
```

All key values are hex strings (e.g. `"A1B2C3D4..."`) stored as `String`.

## Shared Utilities (`sync/mod.rs`)

- `mac_to_colon_format(mac)` → `"AA:BB:CC:DD:EE:FF"`
- `mac_to_flat_format(mac)` → `"AABBCCDDEEFF"`
- `hex_to_bytes(hex_str)` → `Vec<u8>`
- `validate_hex_key(key)` → checks even-length hex

## Elevated Utilities (`elevated.rs`)

- `run_elevated(program, args)` → runs with sudo (askpass) or pkexec fallback
- `is_elevated()` → checks if already root
- `relative_command_path(name)` → path to sibling binary
