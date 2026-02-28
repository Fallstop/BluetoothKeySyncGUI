use serde::{Deserialize, Serialize};

use crate::BluetoothData;

/// A single operation in a batch request to the elevated scrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum BatchOperation {
    WriteKeys {
        controller: String,
        device: String,
        /// Base64-encoded JSON of BluetoothDevice
        data: String,
    },
    DeleteDevice {
        controller: String,
        device: String,
    },
    RestartBluetooth,
    Scan,
}

/// A batch request containing multiple operations to execute sequentially.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequest {
    pub operations: Vec<BatchOperation>,
}

/// Result of a single operation within a batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperationResult {
    pub index: usize,
    pub success: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Response from a batch execution, including per-operation results and optional scan data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResponse {
    pub results: Vec<BatchOperationResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scan_data: Option<BluetoothData>,
}
