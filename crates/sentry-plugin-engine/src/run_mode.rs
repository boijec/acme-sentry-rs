use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DiscoveryType {
    Directory,
    SoloPlugin,
}

impl Into<String> for DiscoveryType {
    fn into(self) -> String {
        match self {
            DiscoveryType::Directory => "Directory".to_string(),
            DiscoveryType::SoloPlugin => "SoloPlugin".to_string(),
        }
    }
}

impl Into<DiscoveryType> for String {
    fn into(self) -> DiscoveryType {
        match self.as_ref() {
            "Directory" => DiscoveryType::Directory,
            "SoloPlugin" => DiscoveryType::SoloPlugin,
            _ => DiscoveryType::SoloPlugin,
        }
    }
}

impl Into<String> for &DiscoveryType {
    fn into(self) -> String {
        match self {
            DiscoveryType::Directory => "Directory".to_string(),
            DiscoveryType::SoloPlugin => "SoloPlugin".to_string(),
        }
    }
}

impl DiscoveryType {
    pub fn as_ds_type(string_type: &str) -> Option<Self> {
        match string_type {
            "Directory" => Some(DiscoveryType::Directory),
            "SoloPlugin" => Some(DiscoveryType::SoloPlugin),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum RunMode {
    Set,
    #[allow(unused)]
    Unset,
}
