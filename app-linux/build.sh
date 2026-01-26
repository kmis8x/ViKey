#!/bin/bash
# Build ViKey IBus engine for Linux
# Usage: ./build.sh [debug|release]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BUILD_TYPE="${1:-release}"

echo "========================================"
echo "Building ViKey for Linux ($BUILD_TYPE)"
echo "========================================"

# Check dependencies
echo ""
echo "[1/4] Checking dependencies..."

if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo not found. Install from https://rustup.rs/"
    exit 1
fi

if ! pkg-config --exists ibus-1.0; then
    echo "Error: IBus development files not found."
    echo "Install: sudo apt install libibus-1.0-dev  # Debian/Ubuntu"
    echo "         sudo dnf install ibus-devel       # Fedora"
    echo "         sudo pacman -S ibus               # Arch"
    exit 1
fi

if ! pkg-config --exists glib-2.0; then
    echo "Error: GLib development files not found."
    echo "Install: sudo apt install libglib2.0-dev"
    exit 1
fi

# Build Rust core
echo ""
echo "[2/4] Building Rust core..."
"$PROJECT_ROOT/scripts/build-linux.sh"

# Build IBus engine
echo ""
echo "[3/4] Building IBus engine..."

cd "$SCRIPT_DIR"

if [ "$BUILD_TYPE" = "debug" ]; then
    cargo build
    BIN_PATH="target/debug/ibus-engine-vikey"
else
    cargo build --release
    BIN_PATH="target/release/ibus-engine-vikey"
fi

# Create install directories
echo ""
echo "[4/4] Preparing installation files..."

INSTALL_DIR="$SCRIPT_DIR/install"
mkdir -p "$INSTALL_DIR/usr/lib/ibus"
mkdir -p "$INSTALL_DIR/usr/share/ibus/component"

cp "$BIN_PATH" "$INSTALL_DIR/usr/lib/ibus/"
cp "$SCRIPT_DIR/data/vikey.xml" "$INSTALL_DIR/usr/share/ibus/component/"

echo ""
echo "========================================"
echo "Build complete!"
echo "========================================"
echo ""
echo "To install system-wide (requires root):"
echo "  sudo cp install/usr/lib/ibus/ibus-engine-vikey /usr/lib/ibus/"
echo "  sudo cp install/usr/share/ibus/component/vikey.xml /usr/share/ibus/component/"
echo "  ibus restart"
echo ""
echo "To install for current user only:"
echo "  mkdir -p ~/.local/lib/ibus"
echo "  mkdir -p ~/.local/share/ibus/component"
echo "  cp install/usr/lib/ibus/ibus-engine-vikey ~/.local/lib/ibus/"
echo "  sed 's|/usr/lib/ibus|~/.local/lib/ibus|g' data/vikey.xml > ~/.local/share/ibus/component/vikey.xml"
echo "  ibus restart"
echo ""
echo "Then enable ViKey in IBus Preferences or System Settings."
echo ""
