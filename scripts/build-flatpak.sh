#!/bin/bash
set -euo pipefail

# Build a Flatpak bundle locally.
# Prerequisites:
#   flatpak install flathub org.gnome.Platform//47 org.gnome.Sdk//47
#   sudo apt install flatpak-builder  (or equivalent)

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
FLATPAK_DIR="$PROJECT_ROOT/dist/flatpak"
BUILD_DIR="$PROJECT_ROOT/flatpak-build"
REPO_DIR="$PROJECT_ROOT/flatpak-repo"

APP_ID="nz.jmw.bluetooth-key-sync"

echo "=== Bluetooth Key Sync — Flatpak Build ==="

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

# Step 2: Copy support files alongside the manifest (flatpak-builder needs them as relative sources)
cp "$DEB_PATH" "$FLATPAK_DIR/bluetooth-key-sync.deb"
cp "$PROJECT_ROOT/dist/$APP_ID.desktop" "$FLATPAK_DIR/$APP_ID.desktop"
cp "$PROJECT_ROOT/dist/$APP_ID.metainfo.xml" "$FLATPAK_DIR/$APP_ID.metainfo.xml"
cp "$PROJECT_ROOT/static/logo.svg" "$FLATPAK_DIR/logo.svg"

# Step 3: Build with flatpak-builder
echo "Running flatpak-builder..."
flatpak-builder --force-clean "$BUILD_DIR" "$FLATPAK_DIR/$APP_ID.yml"

# Step 4: Export to local repo and create bundle
echo "Exporting to repo..."
flatpak-builder --repo="$REPO_DIR" --force-clean "$BUILD_DIR" "$FLATPAK_DIR/$APP_ID.yml"

echo "Creating .flatpak bundle..."
flatpak build-bundle "$REPO_DIR" "$PROJECT_ROOT/$APP_ID.flatpak" "$APP_ID"

# Clean up copied files
rm -f "$FLATPAK_DIR/bluetooth-key-sync.deb"
rm -f "$FLATPAK_DIR/$APP_ID.desktop"
rm -f "$FLATPAK_DIR/$APP_ID.metainfo.xml"
rm -f "$FLATPAK_DIR/logo.svg"

echo ""
echo "=== Done! ==="
echo "Bundle: $PROJECT_ROOT/$APP_ID.flatpak"
echo ""
echo "Install with:"
echo "  flatpak install $PROJECT_ROOT/$APP_ID.flatpak"
echo "Run with:"
echo "  flatpak run $APP_ID"
