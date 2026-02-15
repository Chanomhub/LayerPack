import ctypes
import os
import sys

# Load the shared library
lib_path = os.path.abspath("target/release/liblayer_pack.so")
if not os.path.exists(lib_path):
    # Try .dll for Windows (though we are on Linux) or .dylib for macOS
    lib_path = os.path.abspath("target/release/layer_pack.dll")
    if not os.path.exists(lib_path):
         lib_path = os.path.abspath("target/release/liblayer_pack.dylib")

if not os.path.exists(lib_path):
    print(f"Library not found at {lib_path}")
    sys.exit(1)

lib = ctypes.CDLL(lib_path)

# Define function signature
lib.ffi_unpack_files.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
lib.ffi_unpack_files.restype = ctypes.c_int

# Create a dummy pack for testing (or use existing one)
# We'll rely on 'demo.pack' if it exists, since list_dir showed it.
pack_path = b"demo.pack"
output_dir = b"test_ffi_output"

if not os.path.exists("demo.pack"):
    print("demo.pack not found, cannot test unpacking.")
    # We could try to create one using the CLI, but let's see.
    sys.exit(0)

print(f"Unpacking {pack_path} to {output_dir}...")
result = lib.ffi_unpack_files(pack_path, output_dir)

if result == 0:
    print("Unpack successful!")
else:
    print(f"Unpack failed with code {result}")
