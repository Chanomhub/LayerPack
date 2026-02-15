use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::Path;
use anyhow::anyhow;
use crate::format::{PackManifest, FileEntry, CompressionType};
use sha2::Digest;
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit};

// ดึงรหัสผ่านมาจาก Environment ตอน build
const ENCRYPTION_KEY: &str = env!("LPACK_ENCRYPTION_KEY");

pub trait PackReader: Read + Seek + Send {}
impl<T: Read + Seek + Send> PackReader for T {}

pub struct LoadedPack {
    reader: Box<dyn PackReader>,
    pub manifest: PackManifest,
    entries: HashMap<String, FileEntry>,
    _source_info: String,
}

impl LoadedPack {
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let file = File::open(path)?;
        Self::load_from_reader(Box::new(file), path_str)
    }

    pub fn load_from_memory(data: Vec<u8>) -> anyhow::Result<Self> {
        let cursor = Cursor::new(data);
        Self::load_from_reader(Box::new(cursor), "memory".to_string())
    }

    pub fn load_from_reader(mut reader: Box<dyn PackReader>, source_info: String) -> anyhow::Result<Self> {
        // 1. Check Magic
        let mut magic = [0u8; 5];
        reader.read_exact(&mut magic)?;
        if &magic != b"LPACK" {
            return Err(anyhow!("Invalid pack file format"));
        }

        // 2. Check Version
        let mut version_bytes = [0u8; 4];
        reader.read_exact(&mut version_bytes)?;
        let _version = u32::from_le_bytes(version_bytes);

        // 3. Read Manifest
        let mut manifest_len_bytes = [0u8; 4];
        reader.read_exact(&mut manifest_len_bytes)?;
        let manifest_len = u32::from_le_bytes(manifest_len_bytes) as usize;
        
        let mut manifest_buf = vec![0u8; manifest_len];
        reader.read_exact(&mut manifest_buf)?;
        let manifest: PackManifest = serde_json::from_slice(&manifest_buf)?;

        // 4. Read Index Pointers
        let mut index_offset_bytes = [0u8; 8];
        reader.read_exact(&mut index_offset_bytes)?;
        let index_offset = u64::from_le_bytes(index_offset_bytes);

        let mut index_len_bytes = [0u8; 4];
        reader.read_exact(&mut index_len_bytes)?;
        let index_len = u32::from_le_bytes(index_len_bytes) as usize;

        // 5. Read Index
        reader.seek(SeekFrom::Start(index_offset))?;
        let mut index_buf = vec![0u8; index_len];
        reader.read_exact(&mut index_buf)?;
        let entry_list: Vec<FileEntry> = serde_json::from_slice(&index_buf)?;

        let mut entries = HashMap::new();
        for entry in entry_list {
            entries.insert(entry.path.clone(), entry);
        }

        Ok(Self {
            reader,
            manifest,
            entries,
            _source_info: source_info,
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
        
        self.reader.seek(SeekFrom::Start(entry.offset))?;
        let mut raw_data = vec![0u8; entry.compressed_size as usize];
        self.reader.read_exact(&mut raw_data)?;

        // --- ระบบตรวจสอบและถอดรหัส (Backward Compatible) ---
        let decrypted_data = match entry.encryption {
            crate::format::EncryptionType::Aes256Gcm => {
                if raw_data.len() < 12 {
                    return Err(anyhow!("Invalid encrypted data structure"));
                }
                let (nonce_bytes, ciphertext) = raw_data.split_at(12);
                let mut hasher = sha2::Sha256::new();
                hasher.update(ENCRYPTION_KEY.as_bytes());
                let key_hash = hasher.finalize();
                
                let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key_hash));
                let nonce = Nonce::from_slice(nonce_bytes);

                cipher.decrypt(nonce, ciphertext)
                    .map_err(|_| anyhow!("Decryption failed! Invalid key or corrupted data."))?
            },
            crate::format::EncryptionType::None => {
                // ถ้าไม่ได้เข้ารหัส (ไฟล์เก่า) ให้ใช้ข้อมูลดิบเลย
                raw_data
            }
        };

        match entry.compression {
            CompressionType::Store => Ok(decrypted_data),
            CompressionType::Zstd => {
                let decoded = zstd::stream::decode_all(std::io::Cursor::new(decrypted_data))?;
                Ok(decoded)
            },
            CompressionType::Lz4 => {
                let decoded = lz4_flex::decompress_size_prepended(&decrypted_data)
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
        for pack in &mut self.packs {
            if pack.entries.contains_key(path) {
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
