use std::path::Path;
use bluetooth_model::{BluetoothLowEnergyKey, DeviceID, LinkKey};
use ini::Ini;
use mac_address::MacAddress;
// use bluetooth_model::{DeviceID, LinkKey, LongTermKey, SignatureKey, IdentityResolvingKey, BluetoothLowEnergyData};

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    ini: Ini,
    address: MacAddress,
}


impl DeviceInfo {
    pub fn load_from_file<P: AsRef<Path>>(path: P, address: MacAddress) -> Result<Self, Box<dyn std::error::Error>> {
        let ini = Ini::load_from_file(path)?;
        Ok(DeviceInfo { ini, address })
    }

    pub fn new(address: MacAddress) -> Self {
        DeviceInfo {
            ini: Ini::new(),
            address,
        }
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        self.ini.write_to_file(path)?;
        Ok(())
    }

    pub fn address(&self) -> MacAddress {
        self.address
    }

    pub fn name(&self) -> Option<String> {
        let general = self.ini.section(Some("General"))?;

        if let Some(name) = general.get("Name") {
            if !name.trim().is_empty() {
                return Some(name.to_string());
            }
        }

        if let Some(alias) = general.get("Alias") {
            if !alias.trim().is_empty() {
                return Some(alias.to_string());
            }
        }

        None
    }

    fn set_option_helper(&mut self, section_name: &str, key: &str, value: Option<String>) {
        if let Some(v) = value {
            self.ini.with_section(Some(section_name)).set(key, v);
        } else {
            if let Some(section) = self.ini.section_mut(Some(section_name)) {
                section.remove(key);
            }
        }
    }

    pub fn set_name(&mut self, name: Option<String>) {
        self.set_option_helper("General", "Name", name);
    }

    pub fn alias(&self) -> Option<String> {
        self.ini.section(Some("General"))?
            .get("Alias")
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.to_string())
    }

    pub fn set_alias(&mut self, alias: Option<String>) {
        self.set_option_helper("General", "Alias", alias);
    }

    pub fn device_id(&self) -> Option<DeviceID> {
        let section = self.ini.section(Some("DeviceID"))?;

        Some(DeviceID {
            source: section.get("Source").and_then(|s| s.parse().ok()),
            vendor: section.get("Vendor").and_then(|s| s.parse().ok()),
            product: section.get("Product").and_then(|s| s.parse().ok()),
            version: section.get("Version").and_then(|s| s.parse().ok()),
        })
    }

    pub fn set_device_id(&mut self, device_id: Option<DeviceID>) {
        if let Some(id) = device_id {
            self.set_option_helper("DeviceID", "Source", id.source.map(|v| v.to_string()));
            self.set_option_helper("DeviceID", "Vendor", id.vendor.map(|v| v.to_string()));
            self.set_option_helper("DeviceID", "Product", id.product.map(|v| v.to_string()));
            self.set_option_helper("DeviceID", "Version", id.version.map(|v| v.to_string()));
        } else {
            self.ini.delete(Some("DeviceID"));
        }
    }

    pub fn link_key(&self) -> Option<LinkKey> {
        let section = self.ini.section(Some("LinkKey"))?;
        let key = section.get("Key")?.to_string();

        Some(LinkKey {
            key,
            key_type: section.get("Type").and_then(|s| s.parse().ok()),
            pin_length: section.get("PINLength").and_then(|s| s.parse().ok()),
        })
    }

    pub fn set_link_key(&mut self, link_key: Option<LinkKey>) {
        if let Some(key) = link_key {
            self.ini.with_section(Some("LinkKey")).set("Key", &key.key);
            self.set_option_helper("LinkKey", "Type", key.key_type.map(|v| v.to_string()));
            self.set_option_helper("LinkKey", "PINLength", key.pin_length.map(|v| v.to_string()));
        } else {
            self.ini.delete(Some("LinkKey"));
        }
    }

    pub fn le_pairing_data(&self) -> Option<BluetoothLowEnergyKey> {
        //  {
        //     long_term_key: self.parse_long_term_key("LongTermKey"),
        //     peripheral_long_term_key: self.parse_long_term_key("PeripheralLongTermKey"),
        //     slave_long_term_key: self.parse_long_term_key("SlaveLongTermKey"),
        //     local_signature_key: self.parse_signature_key("LocalSignatureKey"),
        //     remote_signature_key: self.parse_signature_key("RemoteSignatureKey"),
        //     identity_resolving_key: self.parse_identity_resolving_key("IdentityResolvingKey"),
        // }

        // We want to select the most relevant LTK section based on the presence of keys.
        let mut options = [
            self.parse_long_term_key("LongTermKey"),
            self.parse_long_term_key("PeripheralLongTermKey"),
            self.parse_long_term_key("SlaveLongTermKey"),
        ];
        options.sort_by_key(|x| {
            x.as_ref().map_or(0, |key| key.rank_validity())
        });

        // Select the first valid LTK section
        let long_term_key = options.into_iter().find_map(|x| x);

        if long_term_key.is_none() {
            return None;
        }
        let long_term_key = long_term_key.unwrap();

        let local_signature_key = self.parse_signature_key("LocalSignatureKey");
        let remote_signature_key = self.parse_signature_key("RemoteSignatureKey");
        let identity_resolving_key = self.parse_identity_resolving_key("IdentityResolvingKey");

        Some(BluetoothLowEnergyKey {
            long_term_key: long_term_key.long_term_key,
            key_length: long_term_key.key_length,
            ediv: long_term_key.ediv,
            rand: long_term_key.rand,
            identity_resolving_key,
            local_signature_key: local_signature_key.or(remote_signature_key),
        })
    }

    pub fn set_le_pairing_data(&mut self, data: BluetoothLowEnergyKey) {
        let has_ltk_section = self.ini.section(Some("LongTermKey")).is_some();
        let has_peripheral_ltk_section = self.ini.section(Some("PeripheralLongTermKey")).is_some();
        let has_slave_ltk_section = self.ini.section(Some("SlaveLongTermKey")).is_some();

        let has_any_ltk_section = has_ltk_section || has_peripheral_ltk_section || has_slave_ltk_section;

        if has_any_ltk_section {
            if has_ltk_section {
                self.set_long_term_key("LongTermKey", Some(data.clone()));
            }
            if has_peripheral_ltk_section {
                self.set_long_term_key("PeripheralLongTermKey", Some(data.clone()));
            }
            if has_slave_ltk_section {
                self.set_long_term_key("SlaveLongTermKey", Some(data.clone()));
            }
        } else {
            self.set_long_term_key("LongTermKey", Some(data.clone()));
        }

        if data.local_signature_key.is_some() || self.ini.section(Some("LocalSignatureKey")).is_some() {
            self.set_signature_key("LocalSignatureKey", &data.local_signature_key);
            self.set_signature_key("RemoteSignatureKey", &data.local_signature_key);
        }
        if data.identity_resolving_key.is_some() || self.ini.section(Some("IdentityResolvingKey")).is_some() {
            self.set_identity_resolving_key("IdentityResolvingKey", data.identity_resolving_key);
        }
    }

    fn parse_long_term_key(&self, section_name: &str) -> Option<BluetoothLowEnergyKey> {
        let section = self.ini.section(Some(section_name))?;
        let key = section.get("Key").map(|s| s.to_string());

        Some(BluetoothLowEnergyKey {
            long_term_key: key,
            key_length: section.get("EncSize").and_then(|s| s.parse().ok()),
            ediv: section.get("EDiv").and_then(|s| s.parse().ok()),
            rand: section.get("Rand").and_then(|s| s.parse().ok()),
            identity_resolving_key: None,
            local_signature_key: None,
        })
    }

    fn set_long_term_key(&mut self, section_name: &str, ltk: Option<BluetoothLowEnergyKey>) {
        if let Some(key) = ltk.filter(|key| key.long_term_key.is_some()) {

            self.ini.with_section(Some(section_name)).set("Key", &key.long_term_key.unwrap());
            self.set_option_helper(section_name, "EncSize", key.key_length.map(|v| v.to_string()));
            self.set_option_helper(section_name, "EDiv", key.ediv.map(|v| v.to_string()));
            self.set_option_helper(section_name, "Rand", key.rand.map(|v| v.to_string()));
        } else {
            self.ini.delete(Some(section_name));
        }
    }

    fn parse_signature_key(&self, section_name: &str) -> Option<String> {
        let section = self.ini.section(Some(section_name))?;
        let key = section.get("Key").map(|s| s.to_string());

        return key;
    }

    fn set_signature_key(&mut self, section_name: &str, sig_key: &Option<String>) {
        if let Some(key) = sig_key {
            self.ini.with_section(Some(section_name)).set("Key", key);
        } else {
            self.ini.delete(Some(section_name));
        }
    }

    fn parse_identity_resolving_key(&self, section_name: &str) -> Option<String> {
        let section = self.ini.section(Some(section_name))?;
        section.get("Key").map(|s| s.to_string())
    }

    fn set_identity_resolving_key(&mut self, section_name: &str, irk: Option<String>) {
        if let Some(key) = irk {
            self.ini.with_section(Some(section_name)).set("Key", &key);
        } else {
            self.ini.delete(Some(section_name));
        }
    }

    pub fn is_trusted(&self) -> Option<bool> {
        self.ini.section(Some("General"))?
            .get("Trusted")
            .and_then(|s| s.parse().ok())
    }

    pub fn set_trusted(&mut self, trusted: Option<bool>) {
        self.set_option_helper("General", "Trusted", trusted.map(|v| v.to_string()));
    }

    pub fn is_blocked(&self) -> Option<bool> {
        self.ini.section(Some("General"))?
            .get("Blocked")
            .and_then(|s| s.parse().ok())
    }

    pub fn set_blocked(&mut self, blocked: Option<bool>) {
        self.set_option_helper("General", "Blocked", blocked.map(|v| v.to_string()));
    }
}
