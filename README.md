# LayerPack

A Layered Asset System for games, implemented in Rust.

This tool allows you to create and manage asset packs that overlay each other.
High priority packs (e.g., specific languages, mods) override lower priority packs (e.g., base game).

## Features

- **Layered Resolution:** Load multiple packs; the system resolves files based on priority.
- **Type-Aware Compression:**
  - **Text:** Zstd (Best for text/json)
  - **Scripts:** LZ4 (Fast decompression)
  - **Media:** Stored (assuming already compressed)
- **Metadata Embedded:** Each pack contains its own metadata (Name, Type, Lang, Priority).

## Usage

### 1. Build the Tool
```bash
cargo build --release
```
Binary will be at `target/release/layer_pack`.

### 2. Create Packs

**Base Game Pack:**
```bash
./layer_pack create assets/base base.pack --name "Base Game" --type base --priority 0
```

**Language Pack (Thai):**
```bash
./layer_pack create assets/text text.pack --name "Thai Lang" --type text --lang th --priority 10
```

### 3. List Pack Contents
```bash
./layer_pack list text.pack
```

### 4. Resolve Assets
Simulate how the game would load a file (`dialog.txt`). The system checks the highest priority pack first.

```bash
./layer_pack resolve --packs base.pack --packs text.pack dialog.txt
```

## Structure

The `.pack` (or `.lpack`) file format:
- **MIME Type:** `application/vnd.layerpack`
- **Header:** Magic `LPACK`, Version.
- **Manifest:** JSON Metadata.
- **Data:** Compressed file blobs.
- **Index:** Directory of file offsets and sizes.
