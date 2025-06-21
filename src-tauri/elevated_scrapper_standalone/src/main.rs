use bluetooth_model::{BluetoothData, HostDistributions};

mod info_model;
mod scan_filesystem;

use clap::Parser;

/// Interface for elevated scrapper standalone
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Action to take
    #[arg(global = true)]
    task: Option<String>,

    // Number of times to greet
    #[arg(short, long)]
    privileged: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();


    if !args.privileged {
        // Makes it easy to develop without needing to run it explicitly as root
        sudo::escalate_if_needed()?;
    }

    let data = read_bluetooth_data();

    let json_output = serde_json::to_string(&data).expect("Failed to serialize data");
    print!("{}", json_output);

    Ok(())
}

fn read_bluetooth_data() -> BluetoothData {
    let controllers = scan_filesystem::scan_filesystem().unwrap();
    BluetoothData {
        host: HostDistributions::Linux,
        controllers,
        utc_timestamp: chrono::Utc::now(),
        source_path: "/var/lib/bluetooth/".to_string(),
    }
}
