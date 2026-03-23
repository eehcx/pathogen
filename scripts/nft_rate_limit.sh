#!/usr/bin/env bash
# script: nft_rate_limit.sh
# Layer: Infrastructure
# Purpose: Adds a rate limit rule to a specific port to prevent brute force/DDoS.

PROTOCOL=$1
PORT=$2
RATE=$3
UNIT=$4

if [ -z "$PROTOCOL" ] || [ -z "$PORT" ] || [ -z "$RATE" ] || [ -z "$UNIT" ]; then
    echo "Usage: $0 <protocol> <port> <rate> <second|minute>"
    exit 1
fi

nft add table inet filter
nft add chain inet filter input '{ type filter hook input priority 0; policy accept; }'

# Add rate limit rule using a dynamic meter/set based on source IP
nft add rule inet filter input $PROTOCOL dport $PORT meter "pathogen-meter-$PROTOCOL-$PORT" '{ ip saddr limit rate over $RATE/$UNIT }' log prefix \"pathogen-ratelimit: \" counter drop comment \"tui-ratelimit-$PROTOCOL-$PORT-$RATE-$UNIT\"
