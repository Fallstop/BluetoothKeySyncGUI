#!/bin/bash
set -euo pipefail

# Build a Snap package locally.
# Prerequisites:
#   sudo snap install snapcraft --classic

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SNAP_DIR="$PROJECT_ROOT/dist/snap"

echo "=== Bluetooth Key Sync — Snap Build ==="

# Step 1: Build the .deb if it doesn't exist
DEB_PATH=$(find "$PROJECT_ROOT/src-tauri/target/release/bundle/deb/" -name '*.deb' 2>/dev/null | head -1)
if [ -z "$DEB_PATH" ]; then
    echo "No .deb found, building with 'pnpm tauri build'..."
    cd "$PROJECT_ROOT"
    pnpm tauri build
    DEB_PATH=$(find "$PROJECT_ROOT/src-tauri/target/release/bundle/deb/" -name '*.deb' | head -1)
fi

if [ -z "$DEB_PATH" ]; then
    echo "Error: Failed to find .deb after build"
    exit 1
fi

echo "Using .deb: $DEB_PATH"

# Step 2: Copy support files alongside the snapcraft.yaml
cp "$DEB_PATH" "$SNAP_DIR/bluetooth-key-sync.deb"
cp "$PROJECT_ROOT/dist/nz.jmw.bluetooth-key-sync.desktop" "$SNAP_DIR/nz.jmw.bluetooth-key-sync.desktop"
cp "$PROJECT_ROOT/static/logo.svg" "$SNAP_DIR/logo.svg"

# Step 3: Build the snap
echo "Running snapcraft pack..."
cd "$SNAP_DIR"

# Use --destructive-mode if LXD is not available (builds directly on host).
# In CI, the snapcore/action-build action handles the build environment.
if lxc list &>/dev/null; then
    snapcraft pack
else
    echo "LXD not available, using --destructive-mode"
    snapcraft pack --destructive-mode
fi

# Step 4: Move the .snap to project root
SNAP_FILE=$(find "$SNAP_DIR" -name '*.snap' -maxdepth 1 | head -1)
if [ -n "$SNAP_FILE" ]; then
    mv "$SNAP_FILE" "$PROJECT_ROOT/"
    echo ""
    echo "=== Done! ==="
    echo "Snap: $PROJECT_ROOT/$(basename "$SNAP_FILE")"
    echo ""
    echo "Install with:"
    echo "  sudo snap install --dangerous $PROJECT_ROOT/$(basename "$SNAP_FILE")"
else
    echo "Error: No .snap file produced"
    exit 1
fi

# Clean up copied files
rm -f "$SNAP_DIR/bluetooth-key-sync.deb"
rm -f "$SNAP_DIR/nz.jmw.bluetooth-key-sync.desktop"
rm -f "$SNAP_DIR/logo.svg"
