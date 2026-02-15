use serde::{Deserialize, Serialize};

pub const CONTENT_TYPE: &str = "application/vnd.layerpack";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PackType {
    Base,
    Text,
    Image,
    Audio,
    Script,
    Mod,
    Other,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CompressionType {
    Store,
    Zstd,
    Lz4,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PackManifest {
    pub name: String,
    #[serde(rename = "type")]
    pub pack_type: PackType,
    pub lang: Option<String>,
    #[serde(default)]
    pub priority: i32,
    pub description: Option<String>,
    pub version: Option<String>,
    #[serde(default)]
    pub custom_ref: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EncryptionType {
    None,
    Aes256Gcm,
}

impl Default for EncryptionType {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileEntry {
    pub path: String,
    pub offset: u64,
    pub original_size: u64,
    pub compressed_size: u64,
    pub compression: CompressionType,
    #[serde(default)]
    pub encryption: EncryptionType,
    pub hash: String, 
}
