use std::{error::Error, path::Path};
use nt_hive2::*;
use std::fs::File; // Changed from tokio::fs::File
use tokio::task; // Added for spawn_blocking
use crate::bluetooth::bluetooth_data::BluetoothData;

pub async fn extract_hive_data(hive_path: &Path) -> Result<BluetoothData, Box<dyn std::error::Error + Send + Sync>> {
    let path_buf = hive_path.to_path_buf();

    // Use spawn_blocking to run the blocking code in a separate thread
    let bluetooth_data = task::spawn_blocking(move || -> Result<BluetoothData, Box<dyn std::error::Error + Send + Sync>> {
        let hive_file = File::open(path_buf)?;
        let hive_result = Hive::new(hive_file, HiveParseMode::NormalWithBaseBlock);
        if let Err(e) = hive_result {
          let message = format!("Failed to open hive file: {}", e);
          return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, message)));
        }
        let mut hive = hive_result.unwrap();

        let root_key = hive.root_key_node()?;

        for sk in root_key.subkeys(&mut hive)?.iter() {
            println!("\n[{}]; last written: {}", sk.borrow().name(), sk.borrow().timestamp());
            for value in sk.borrow().values() {
                println!("\"{}\" = {}", value.name(), value.value());
            }
        }
        Ok(BluetoothData::default())
    }).await??;

    Ok(bluetooth_data)
}
