#!/bin/bash
# Build ViKey for macOS
# Usage: ./build.sh [debug|release]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
APP_DIR="$SCRIPT_DIR/ViKey"
BUILD_TYPE="${1:-release}"

echo "========================================"
echo "Building ViKey for macOS ($BUILD_TYPE)"
echo "========================================"

# Step 1: Build Rust core
echo ""
echo "[1/3] Building Rust core..."
"$PROJECT_ROOT/scripts/build-macos.sh"

# Check if library exists
LIB_PATH="$APP_DIR/lib/libvikey_core.a"
if [ ! -f "$LIB_PATH" ]; then
    echo "Error: Rust library not found at $LIB_PATH"
    exit 1
fi

# Step 2: Build Swift app
echo ""
echo "[2/3] Building Swift app..."

BUILD_DIR="$SCRIPT_DIR/build"
OUTPUT_DIR="$BUILD_DIR/ViKey.app"

mkdir -p "$BUILD_DIR"

# Compile Swift sources
SOURCES=(
    "$APP_DIR/Sources/main.swift"
    "$APP_DIR/Sources/RustBridge.swift"
    "$APP_DIR/Sources/ViKeyInputController.swift"
    "$APP_DIR/Sources/Settings.swift"
)

FRAMEWORKS="-framework Foundation -framework AppKit -framework InputMethodKit"
BRIDGE_HEADER="$APP_DIR/Sources/ViKey-Bridging-Header.h"

if [ "$BUILD_TYPE" = "debug" ]; then
    OPT_FLAGS="-g -Onone"
else
    OPT_FLAGS="-O"
fi

# Create app bundle structure
mkdir -p "$OUTPUT_DIR/Contents/MacOS"
mkdir -p "$OUTPUT_DIR/Contents/Resources"

# Compile
swiftc \
    $OPT_FLAGS \
    -import-objc-header "$BRIDGE_HEADER" \
    -L "$APP_DIR/lib" \
    -lvikey_core \
    $FRAMEWORKS \
    -target arm64-apple-macos12.0 \
    -o "$OUTPUT_DIR/Contents/MacOS/ViKey" \
    "${SOURCES[@]}"

# Copy Info.plist and resources
cp "$APP_DIR/Resources/Info.plist" "$OUTPUT_DIR/Contents/"
cp "$APP_DIR/Resources/AppIcon.icns" "$OUTPUT_DIR/Contents/Resources/" 2>/dev/null || true
cp "$APP_DIR/Resources/ViKey.tiff" "$OUTPUT_DIR/Contents/Resources/" 2>/dev/null || true

# Step 3: Sign (development)
echo ""
echo "[3/3] Code signing..."
codesign --force --sign - "$OUTPUT_DIR"

echo ""
echo "========================================"
echo "Build complete!"
echo "========================================"
echo ""
echo "Output: $OUTPUT_DIR"
echo ""
echo "To install:"
echo "  cp -r '$OUTPUT_DIR' ~/Library/Input\\ Methods/"
echo ""
echo "Then:"
echo "  1. System Preferences → Keyboard → Input Sources"
echo "  2. Click + → Add ViKey"
echo "  3. Select ViKey from input menu"
echo ""
