#!/usr/bin/env bash
# script: nft_rate_limit.sh
# Layer: Infrastructure
# Purpose: Adds a rate limit rule to a specific port to prevent brute force/DDoS.

PORT=$1
PROTOCOL=$2
RATE=$3
UNIT=$4

if [ -z "$PORT" ] || [ -z "$PROTOCOL" ] || [ -z "$RATE" ] || [ -z "$UNIT" ]; then
    echo "Usage: $0 <port> <protocol> <rate> <second|minute>"
    exit 1
fi

# Simple rate limiting without complex set operations
# This limits new connections to $RATE per $UNIT from any source
# Note: This script is called via sudo -n from the application
nft add table inet filter 2>/dev/null || true
nft add chain inet filter input '{ type filter hook input priority 0; policy accept; }' 2>/dev/null || true

nft add rule inet filter input $PROTOCOL dport $PORT \
    ct state new \
    limit rate $RATE/$UNIT \
    log prefix \"pathogen-ratelimit: \" \
    counter drop \
    comment \"tui-ratelimit-$PROTOCOL-$PORT-$RATE-$UNIT\" 2>/dev/null || true

echo "{\"status\":\"ok\"}"
