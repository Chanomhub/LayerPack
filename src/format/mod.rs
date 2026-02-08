use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileEntry {
    pub path: String,
    pub offset: u64,
    pub original_size: u64,
    pub compressed_size: u64,
    pub compression: CompressionType,
    pub hash: String, 
}
