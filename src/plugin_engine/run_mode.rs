use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DiscoveryType {
    Directory,
    SoloPlugin,
}

impl DiscoveryType {
    pub fn as_str(&self) -> &str {
        match self {
            DiscoveryType::Directory => "Directory",
            DiscoveryType::SoloPlugin => "SoloPlugin",
        }
    }
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
