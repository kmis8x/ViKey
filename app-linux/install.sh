#!/bin/bash
# ViKey Linux Installation Script

set -e

echo "Installing ViKey IBus Engine..."

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "Please run with sudo: sudo ./install.sh"
    exit 1
fi

# Install IBus engine binary
install -Dm755 ibus-engine-vikey /usr/lib/ibus/ibus-engine-vikey

# Install component descriptor
install -Dm644 vikey.xml /usr/share/ibus/component/vikey.xml

# Restart IBus daemon
if command -v ibus &> /dev/null; then
    ibus write-cache
    echo ""
    echo "Installation complete!"
    echo ""
    echo "To enable ViKey:"
    echo "1. Run: ibus restart"
    echo "2. Open Settings -> Keyboard -> Input Sources"
    echo "3. Add Vietnamese -> ViKey"
    echo ""
else
    echo "IBus not found. Please install IBus first:"
    echo "  Ubuntu/Debian: sudo apt install ibus"
    echo "  Fedora: sudo dnf install ibus"
    echo "  Arch: sudo pacman -S ibus"
fi
