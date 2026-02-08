use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use anyhow::anyhow;
use crate::format::{PackManifest, FileEntry, CompressionType};

pub struct LoadedPack {
    file: File,
    pub manifest: PackManifest,
    entries: HashMap<String, FileEntry>,
    _path: String,
}

impl LoadedPack {
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let mut file = File::open(path)?;
        
        // 1. Check Magic
        let mut magic = [0u8; 5];
        file.read_exact(&mut magic)?;
        if &magic != b"LPACK" {
            return Err(anyhow!("Invalid pack file format"));
        }

        // 2. Check Version
        let mut version_bytes = [0u8; 4];
        file.read_exact(&mut version_bytes)?;
        let _version = u32::from_le_bytes(version_bytes);

        // 3. Read Manifest
        let mut manifest_len_bytes = [0u8; 4];
        file.read_exact(&mut manifest_len_bytes)?;
        let manifest_len = u32::from_le_bytes(manifest_len_bytes) as usize;
        
        let mut manifest_buf = vec![0u8; manifest_len];
        file.read_exact(&mut manifest_buf)?;
        let manifest: PackManifest = serde_json::from_slice(&manifest_buf)?;

        // 4. Read Index Pointers
        let mut index_offset_bytes = [0u8; 8];
        file.read_exact(&mut index_offset_bytes)?;
        let index_offset = u64::from_le_bytes(index_offset_bytes);

        let mut index_len_bytes = [0u8; 4];
        file.read_exact(&mut index_len_bytes)?;
        let index_len = u32::from_le_bytes(index_len_bytes) as usize;

        // 5. Read Index
        file.seek(SeekFrom::Start(index_offset))?;
        let mut index_buf = vec![0u8; index_len];
        file.read_exact(&mut index_buf)?;
        let entry_list: Vec<FileEntry> = serde_json::from_slice(&index_buf)?;

        let mut entries = HashMap::new();
        for entry in entry_list {
            entries.insert(entry.path.clone(), entry);
        }

        Ok(Self {
            file,
            manifest,
            entries,
            _path: path_str,
        })
    }

    pub fn get_entry(&self, path: &str) -> Option<&FileEntry> {
        self.entries.get(path)
    }

    pub fn file_list(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }

    pub fn read_file(&mut self, path: &str) -> anyhow::Result<Vec<u8>> {
        let entry = self.entries.get(path).ok_or_else(|| anyhow!("File not found in pack"))?;
        
        self.file.seek(SeekFrom::Start(entry.offset))?;
        let mut compressed_data = vec![0u8; entry.compressed_size as usize];
        self.file.read_exact(&mut compressed_data)?;

        match entry.compression {
            CompressionType::Store => Ok(compressed_data),
            CompressionType::Zstd => {
                let decoded = zstd::stream::decode_all(std::io::Cursor::new(compressed_data))?;
                Ok(decoded)
            },
            CompressionType::Lz4 => {
                let decoded = lz4_flex::decompress_size_prepended(&compressed_data)
                    .map_err(|e| anyhow!("LZ4 Decompression error: {}", e))?;
                Ok(decoded)
            },
        }
    }
}

pub struct Resolver {
    packs: Vec<LoadedPack>,
}

impl Resolver {
    pub fn new() -> Self {
        Self { packs: Vec::new() }
    }

    pub fn add_pack(&mut self, pack: LoadedPack) {
        self.packs.push(pack);
        // Sort by priority (descending)
        self.packs.sort_by(|a, b| b.manifest.priority.cmp(&a.manifest.priority));
    }

    pub fn resolve(&mut self, path: &str) -> Option<Vec<u8>> {
        // Check packs in order of priority
        for pack in &mut self.packs {
            if pack.entries.contains_key(path) {
                // Found!
                 // Note: We need mutable access to read file (because of seek), 
                 // but we are iterating.
                 // Actually `read_file` needs mut self because of File seek.
                 // Ideally we should use pread or open a new handle, but for this CLI tool, 
                 // we can refactor slightly or just accept we modify state.
                 // Since we return Option<Vec<u8>>, we break immediately.
                 return pack.read_file(path).ok();
            }
        }
        None
    }
    
    pub fn list_layers(&self, path: &str) -> Vec<String> {
        let mut found_in = Vec::new();
        for pack in &self.packs {
            if pack.entries.contains_key(path) {
                found_in.push(format!("{} (Priority: {})", pack.manifest.name, pack.manifest.priority));
            }
        }
        found_in
    }
}
