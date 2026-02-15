#!/bin/bash
set -e

# 1. Build
echo "Building LayerPack..."
cargo build --quiet

LPACK="./target/debug/layer_pack"

# 2. Setup Directories
rm -rf test_env
mkdir -p test_env/source
echo "Hello Unpack" > test_env/source/hello.txt

# 3. Create Pack
echo "Creating Test Pack..."
$LPACK create test_env/source test_env/test.lpack --name "TestPack" --type base

# 4. Unpack
echo "Unpacking..."
$LPACK unpack test_env/test.lpack test_env/output

# 5. Verify
if [ -f "test_env/output/hello.txt" ]; then
    echo "SUCCESS: File unpacked correctly."
    cat test_env/output/hello.txt
else
    echo "FAILURE: File not found."
    exit 1
fi
