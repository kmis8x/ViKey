#!/bin/bash
# Build ViKey Rust core for Linux (x86_64)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CORE_DIR="$PROJECT_ROOT/core"

echo "Building ViKey core for Linux (x86_64)..."

# Check Rust target
if ! rustup target list --installed | grep -q "x86_64-unknown-linux-gnu"; then
    echo "Installing Rust target for Linux x86_64..."
    rustup target add x86_64-unknown-linux-gnu
fi

# Build
cd "$CORE_DIR"
cargo build --release --target x86_64-unknown-linux-gnu

# Output location
OUTPUT="$CORE_DIR/target/x86_64-unknown-linux-gnu/release/libvikey_core.a"

if [ -f "$OUTPUT" ]; then
    echo ""
    echo "Build successful!"
    echo "Static library: $OUTPUT"
    echo ""

    # Copy to app-linux
    LINUX_LIB="$PROJECT_ROOT/app-linux/lib"
    mkdir -p "$LINUX_LIB"
    cp "$OUTPUT" "$LINUX_LIB/"
    echo "Copied to: $LINUX_LIB/libvikey_core.a"
else
    echo "Build failed!"
    exit 1
fi
