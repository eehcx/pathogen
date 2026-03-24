#!/usr/bin/env bash
# script: nft_quarantine_ip.sh
# Layer: Infrastructure
# Purpose: Adds an IP address to the quarantine set (Blacklist).

IP=$1

if [ -z "$IP" ]; then
    echo "Usage: $0 <ip_address>"
    exit 1
fi

# Note: This script is called via sudo -n from the application
nft add table inet filter 2>/dev/null || true
nft add chain inet filter input '{ type filter hook input priority 0; policy accept; }' 2>/dev/null || true

# Ensure the set exists
nft add set inet filter pathogen_quarantine '{ type ipv4_addr; flags interval; }' 2>/dev/null || true

# Ensure the rule that drops traffic from this set exists.
RULE_EXISTS=$(nft -a list chain inet filter input 2>/dev/null | grep "pathogen-quarantine-rule" || true)
if [ -z "$RULE_EXISTS" ]; then
    nft insert rule inet filter input ip saddr @pathogen_quarantine log prefix \"pathogen-quarantine: \" counter drop comment \"pathogen-quarantine-rule\" 2>/dev/null || true
fi

# Add the IP to the set
nft add element inet filter pathogen_quarantine "{ $IP }" 2>/dev/null || true

echo "{\"status\":\"ok\"}"
