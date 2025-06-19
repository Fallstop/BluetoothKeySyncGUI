#!/bin/bash

# Build script for elevated scrapper binary
# This ensures the binary is built and placed in the correct location for Tauri bundling

set -e

echo "Building elevated scrapper binary..."

BASE_DIR="$(dirname "$0")/src-tauri"

cargo rustc --release --bin elevated_scrapper --manifest-path "$BASE_DIR/elevated_scrapper_standalone/Cargo.toml"

TARGET_TRIPLE="x86_64-unknown-linux-gnu"

# Copy the binary to the expected location with platform-specific name
SRC_PATH="$BASE_DIR/target/release/elevated_scrapper"
DST_PATH="$BASE_DIR/target/release/elevated_scrapper-$TARGET_TRIPLE"

if [ -f "$SRC_PATH" ]; then
    cp "$SRC_PATH" "$DST_PATH"
		echo "Binary built and copied to $DST_PATH"
else
		echo "Error: Binary not found at $SRC_PATH"
    exit 1
fi
