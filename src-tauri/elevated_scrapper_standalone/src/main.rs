use std::fs;
use std::path::Path;

use bluetooth_model::BluetoothData;
use serde::Serialize;

mod scan_filesystem;

fn main() -> Result<(), Box<dyn std::error::Error>> {
		// Makes it easy to develop without needing to run it explicitly as root
		sudo::escalate_if_needed()?;

    let data = read_bluetooth_data();

    let json_output = serde_json::to_string(&data).expect("Failed to serialize data");
    println!("{}", json_output);

		Ok(())
}

fn read_bluetooth_data() -> BluetoothData {
    scan_filesystem::scan_filesystem().unwrap();
    BluetoothData::default()
}
