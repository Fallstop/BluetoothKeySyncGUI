use serde::{Deserialize, Serialize};

use crate::BluetoothData;

/// A command sent from the Tauri app to the elevated worker over stdin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerCommand {
    pub id: u64,
    pub op: WorkerOperation,
}

/// The operation to perform.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WorkerOperation {
    Scan,
    WriteKeys {
        controller: String,
        device: String,
        /// Raw JSON string of BluetoothDevice (not base64 — pipe transport doesn't need it)
        data: String,
    },
    DeleteDevice {
        controller: String,
        device: String,
    },
    RestartBluetooth,
    Batch {
        operations: Vec<WorkerOperation>,
    },
    Shutdown,
    Ping,
}

/// A response sent from the elevated worker back to the Tauri app over stdout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerResponse {
    pub id: u64,
    pub result: WorkerResult,
}

/// The result of a worker operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum WorkerResult {
    Ok {
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<WorkerResponseData>,
    },
    Err {
        message: String,
    },
}

/// Data payload for successful responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum WorkerResponseData {
    ScanResult { bluetooth_data: BluetoothData },
    BatchResult {
        results: Vec<BatchItemResult>,
        scan_data: Option<BluetoothData>,
    },
    Pong,
}

/// Result of a single item within a Batch operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItemResult {
    pub index: usize,
    pub success: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// The ready signal sent by the worker on startup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerReady {
    pub ready: bool,
}
