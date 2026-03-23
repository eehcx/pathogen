#!/usr/bin/env bash
# script: nft_block_port.sh
# Layer: Infrastructure (OS boundary)
# Purpose: Adds a rule to block a specific protocol and port.
# Usage: ./nft_block_port.sh <protocol> <port>

PROTOCOL=$1
PORT=$2

if [ -z "$PROTOCOL" ] || [ -z "$PORT" ]; then
    echo "Usage: $0 <protocol> <port>"
    exit 1
fi

# Ensure table inet filter exists
nft add table inet filter

# Ensure input chain exists
nft add chain inet filter input '{ type filter hook input priority 0; policy accept; }'

# Add the drop rule with logging
nft add rule inet filter input $PROTOCOL dport $PORT log prefix \"pathogen-drop: \" counter drop comment \"tui-blocked-$PROTOCOL-$PORT\"
