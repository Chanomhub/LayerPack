use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::Path;
use walkdir::WalkDir;
use sha2::{Sha256, Digest};
use crate::format::{PackManifest, FileEntry, CompressionType, PackType};

pub struct PackBuilder {
    manifest: PackManifest,
}

impl PackBuilder {
    pub fn new(manifest: PackManifest) -> Self {
        Self { manifest }
    }

    pub fn build<P: AsRef<Path>>(&self, source_dir: P, output_file: P) -> anyhow::Result<()> {
        let source_dir = source_dir.as_ref();
        let mut out = File::create(output_file)?;
        
        // 1. Write Header Magic
        out.write_all(b"LPACK")?;
        out.write_all(&1u32.to_le_bytes())?;

        // 2. Prepare Manifest
        let manifest_json = serde_json::to_vec(&self.manifest)?;
        out.write_all(&(manifest_json.len() as u32).to_le_bytes())?;
        out.write_all(&manifest_json)?;

        // 3. Collect Files and Process
        let mut entries = Vec::new();
        // For large packs, we should write data directly to file after index, but we need index first?
        // Actually, usually: Header -> Index -> Data.
        // If we write Data first, we know offsets.
        // Let's write Data first (after placeholder for Index), then go back and write Index.
        
        // Placeholder for Index Position and Size
        let index_ptr_pos = out.stream_position()?;
        out.write_all(&0u64.to_le_bytes())?;
        out.write_all(&0u32.to_le_bytes())?;

        let _data_start_pos = out.stream_position()?;
        
        for entry in WalkDir::new(source_dir).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let path = entry.path();
                let rel_path = path.strip_prefix(source_dir)?.to_string_lossy().replace("\\", "/");
                
                // Skip hidden files or config files if needed
                if rel_path.starts_with(".") || rel_path == "pack.json" {
                    continue;
                }

                let mut file = File::open(path)?;
                let mut content = Vec::new();
                file.read_to_end(&mut content)?;
                
                // Calculate Hash
                let mut hasher = Sha256::new();
                hasher.update(&content);
                let hash = hex::encode(hasher.finalize());

                // Select Compression
                let (compressed_data, compression) = self.compress_data(&content, &rel_path);

                let offset = out.stream_position()?;
                out.write_all(&compressed_data)?;
                
                entries.push(FileEntry {
                    path: rel_path,
                    offset,
                    original_size: content.len() as u64,
                    compressed_size: compressed_data.len() as u64,
                    compression,
                    encryption: crate::format::EncryptionType::None,
                    hash,
                });
            }
        }

        let data_end_pos = out.stream_position()?;

        // 4. Write Index
        let index_json = serde_json::to_vec(&entries)?;
        out.write_all(&index_json)?;
        let index_len = index_json.len();

        // 5. Update Index Pointers
        out.seek(SeekFrom::Start(index_ptr_pos))?;
        out.write_all(&data_end_pos.to_le_bytes())?;
        out.write_all(&(index_len as u32).to_le_bytes())?;

        Ok(())
    }

    fn compress_data(&self, data: &[u8], path: &str) -> (Vec<u8>, CompressionType) {
        // Simple heuristic
        let ext = Path::new(path).extension().and_then(|s| s.to_str()).unwrap_or("");
        
        match self.manifest.pack_type {
            PackType::Text => {
                // Always try Zstd for text pack
                let compressed = zstd::stream::encode_all(std::io::Cursor::new(data), 3).unwrap_or(data.to_vec());
                if compressed.len() < data.len() {
                    (compressed, CompressionType::Zstd)
                } else {
                    (data.to_vec(), CompressionType::Store)
                }
            },
            PackType::Script => {
                // Scripts use LZ4
                if ext == "lua" || ext == "js" || ext == "py" {
                     let compressed = lz4_flex::compress_prepend_size(data);
                     if compressed.len() < data.len() {
                        (compressed, CompressionType::Lz4)
                     } else {
                        (data.to_vec(), CompressionType::Store)
                     }
                } else {
                    (data.to_vec(), CompressionType::Store)
                }
            },
            _ => {
                // Default heuristic
                 match ext {
                    "txt" | "json" | "xml" | "yaml" | "csv" => {
                        let compressed = zstd::stream::encode_all(std::io::Cursor::new(data), 3).unwrap_or(data.to_vec());
                        if compressed.len() < data.len() {
                            (compressed, CompressionType::Zstd)
                        } else {
                            (data.to_vec(), CompressionType::Store)
                        }
                    },
                    _ => (data.to_vec(), CompressionType::Store),
                }
            }
        }
    }
}
