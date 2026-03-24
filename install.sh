#!/usr/bin/env bash
set -e

# ==============================================================================
# Pathogen Installer
# ==============================================================================
# This script compiles the Rust project and installs the binary and shell scripts
# globally. It sets Linux capabilities (cap_net_admin) on the binary to allow
# nftables operations without requiring sudo or password.
# ==============================================================================

echo "Starting Pathogen installation..."

if ! command -v cargo &> /dev/null; then
    echo "❌ Error: 'cargo' (Rust) is not installed. Please install Rust first."
    exit 1
fi

echo "📦 Compiling the Rust project (release mode)..."
cargo build --release

INSTALL_DIR="/usr/local/bin"
SCRIPTS_DIR="/usr/local/share/pathogen/scripts"

echo "Requesting sudo permissions for installation..."
sudo -v

echo "Copying files to system directories..."
sudo mkdir -p "$SCRIPTS_DIR"

sudo cp scripts/*.sh "$SCRIPTS_DIR/"
sudo chmod +x "$SCRIPTS_DIR"/*.sh

sudo cp target/release/pathogen "$INSTALL_DIR/"

OLD_SUDOERS_FILE="/etc/sudoers.d/pathogen"
if [ -f "$OLD_SUDOERS_FILE" ]; then
    echo "Cleaning up old sudoers configuration..."
    sudo rm -f "$OLD_SUDOERS_FILE"
fi

if ! command -v setcap &> /dev/null; then
    echo "❌ Error: 'setcap' is not available. Please install libcap2-bin package."
    echo "   On Debian/Ubuntu: sudo apt install libcap2-bin"
    exit 1
fi

echo "Setting Linux capabilities for pathogen..."
if ! sudo setcap cap_net_admin+ep "$INSTALL_DIR/pathogen" 2>/dev/null; then
    echo "⚠️  Warning: Failed to set capabilities. This may happen in container environments."
    echo "   You may need to run pathogen with elevated privileges manually."
fi

echo "Installation complete!"
echo ""
echo "You can now run the firewall by typing: pathogen"
echo "The TUI will run seamlessly without asking for passwords."
