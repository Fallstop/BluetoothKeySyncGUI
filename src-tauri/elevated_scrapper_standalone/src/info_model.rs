// This file models the structure of the 'info' file found in
// /var/lib/bluetooth/<adapter_address>/<device_address>/info
// The file is in INI format and can be parsed with a crate like `serde_ini`.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
pub struct Info {
    #[serde(rename = "General")]
    pub general: General,
    #[serde(rename = "DeviceID")]
    pub device_id: Option<DeviceID>,
    #[serde(rename = "LinkKey")]
    pub link_key: Option<LinkKey>,
    #[serde(rename = "LongTermKey")]
    pub long_term_key: Option<LongTermKey>,
    #[serde(rename = "PeripheralLongTermKey")]
    pub peripheral_long_term_key: Option<LongTermKey>,
    #[serde(rename = "ConnectionParameters")]
    pub connection_parameters: Option<ConnectionParameters>,
    #[serde(rename = "LocalSignatureKey")]
    pub local_signature_key: Option<SignatureKey>,
    #[serde(rename = "RemoteSignatureKey")]
    pub remote_signature_key: Option<SignatureKey>,
    #[serde(rename = "ServiceChanged")]
    pub service_changed: Option<ServiceChanged>,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct General {
    pub name: Option<String>,
    pub alias: Option<String>,
    pub class: Option<String>,
    pub appearance: Option<String>,
    pub supported_technologies: Option<String>,
    pub address_type: Option<String>,
    pub trusted: Option<bool>,
    pub blocked: Option<bool>,
    pub services: Option<String>,
    pub preferred_bearer: Option<String>,
    pub last_used_bearer: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct DeviceID {
    pub source: Option<u32>,
    pub vendor: Option<u32>,
    pub product: Option<u32>,
    pub version: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct LinkKey {
    pub key: String,
    #[serde(rename = "Type")]
    pub key_type: Option<u8>,
    #[serde(rename = "PINLength")]
    pub pin_length: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct LongTermKey {
    pub key: String,
    pub authenticated: Option<bool>,
    #[serde(rename = "EncSize")]
    pub enc_size: Option<u8>,
    #[serde(rename = "EDiv")]
    pub ediv: Option<u16>,
    pub rand: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct ConnectionParameters {
    pub min_interval: Option<u32>,
    pub max_interval: Option<u32>,
    pub latency: Option<u32>,
    pub timeout: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct SignatureKey {
    pub key: String,
    pub counter: Option<u32>,
    pub authenticated: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
pub struct ServiceChanged {
    #[serde(rename = "CCC_LE")]
    pub ccc_le: Option<u32>,
    #[serde(rename = "CCC_BR/EDR")]
    pub ccc_br_edr: Option<u32>,
}
