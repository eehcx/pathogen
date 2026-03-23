#!/usr/bin/env bash
set -e

# ==============================================================================
# Pathogen Installer
# ==============================================================================
# This script compiles the Rust project and installs the binary and shell scripts
# globally. It also configures sudoers to allow the current user to run the
# necessary nft commands without a password, preventing the TUI from breaking.
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

SUDOERS_FILE="/etc/sudoers.d/pathogen"
USER_GROUP=$(id -gn)

echo "Configuring sudoers in $SUDOERS_FILE..."
sudo bash -c "cat > $SUDOERS_FILE" <<EOF
# Automatically generated for Pathogen
%${USER_GROUP} ALL=(root) NOPASSWD: ${SCRIPTS_DIR}/nft_list_rules.sh
%${USER_GROUP} ALL=(root) NOPASSWD: ${SCRIPTS_DIR}/nft_block_port.sh
%${USER_GROUP} ALL=(root) NOPASSWD: ${SCRIPTS_DIR}/nft_unblock_port.sh
%${USER_GROUP} ALL=(root) NOPASSWD: ${SCRIPTS_DIR}/nft_get_logs.sh
%${USER_GROUP} ALL=(root) NOPASSWD: ${SCRIPTS_DIR}/nft_rate_limit.sh
%${USER_GROUP} ALL=(root) NOPASSWD: ${SCRIPTS_DIR}/nft_quarantine_ip.sh
%${USER_GROUP} ALL=(root) NOPASSWD: ${SCRIPTS_DIR}/nft_unquarantine_ip.sh
%${USER_GROUP} ALL=(root) NOPASSWD: ${SCRIPTS_DIR}/nft_list_quarantine.sh
EOF

sudo chmod 0440 "$SUDOERS_FILE"

echo "Installation complete!"
echo ""
echo "You can now run the firewall by typing: pathogen"
echo "The TUI will run seamlessly without asking for passwords."
