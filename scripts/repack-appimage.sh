#!/bin/bash
set -euo pipefail

# Repack the Tauri-built AppImage using the legacy AppImageKit appimagetool.
# Tauri v2 uses the new type2-runtime which is incompatible with AppImageLauncher
# (unmaintained since 2022). Repacking with the old appimagetool restores compat.
# See: https://github.com/audacity/audacity/issues/8790

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
APPIMAGE_DIR="$PROJECT_ROOT/src-tauri/target/release/bundle/appimage"
APPDIR="$APPIMAGE_DIR/bluetooth-key-sync.AppDir"
CACHE_DIR="${HOME}/.cache/tauri/appimagetool-legacy"
APPIMAGETOOL="$CACHE_DIR/appimagetool-x86_64.AppImage"

echo "=== Bluetooth Key Sync — Repack AppImage (AppImageLauncher compat) ==="

# Step 1: Verify the AppDir exists
if [ ! -d "$APPDIR" ]; then
    echo "Error: AppDir not found at $APPDIR"
    echo "Run 'pnpm tauri build' first."
    exit 1
fi

# Step 2: Download appimagetool if not cached
if [ ! -x "$APPIMAGETOOL" ]; then
    echo "Downloading appimagetool (AppImageKit)..."
    mkdir -p "$CACHE_DIR"
    curl -fSL -o "$APPIMAGETOOL" \
        "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage"
    chmod +x "$APPIMAGETOOL"
fi

# Step 3: Remove Tauri's .AppImage output (built with incompatible type2-runtime)
echo "Removing Tauri-built .AppImage files..."
rm -f "$APPIMAGE_DIR"/*.AppImage

# Step 4: Determine output filename (match Tauri's naming: productName_version_amd64.AppImage)
VERSION=$(grep '"version"' "$PROJECT_ROOT/src-tauri/tauri.conf.json" | head -1 | sed 's/.*: *"\(.*\)".*/\1/')
OUTPUT="$APPIMAGE_DIR/bluetooth-key-sync_${VERSION}_amd64.AppImage"

# Step 5: Repack with legacy appimagetool
echo "Repacking with appimagetool..."
ARCH=x86_64 "$APPIMAGETOOL" --no-appstream "$APPDIR" "$OUTPUT"

echo ""
echo "=== Done! ==="
echo "AppImage: $OUTPUT"
