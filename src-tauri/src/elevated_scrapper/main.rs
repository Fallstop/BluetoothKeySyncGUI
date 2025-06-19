
use std::fs;
use std::path::Path;

use serde::Serialize;

#[derive(Serialize)]
struct BluetoothData {
    placeholder: String,
}

fn main() {
    let data = read_bluetooth_data();

    let json_output = serde_json::to_string(&data).expect("Failed to serialize data");
    println!("{}", json_output);
}

fn read_bluetooth_data() -> BluetoothData {
    BluetoothData {
        placeholder: "e.".to_string(),
    }
}

