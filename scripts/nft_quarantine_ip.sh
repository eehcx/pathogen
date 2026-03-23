#!/usr/bin/env bash
# script: nft_quarantine_ip.sh
# Layer: Infrastructure
# Purpose: Adds an IP address to the quarantine set (Blacklist).

IP=$1

if [ -z "$IP" ]; then
    echo "Usage: $0 <ip_address>"
    exit 1
fi

nft add table inet filter
nft add chain inet filter input '{ type filter hook input priority 0; policy accept; }'

# Ensure the set exists
nft add set inet filter pathogen_quarantine '{ type ipv4_addr; flags interval; }'

# Ensure the rule that drops traffic from this set exists. We use a handle check or just try to add it safely.
# A simple way without failing if it exists:
RULE_EXISTS=$(nft -a list chain inet filter input | grep "pathogen-quarantine-rule" || true)
if [ -z "$RULE_EXISTS" ]; then
    nft insert rule inet filter input ip saddr @pathogen_quarantine log prefix \"pathogen-quarantine: \" counter drop comment \"pathogen-quarantine-rule\"
fi

# Add the IP to the set
nft add element inet filter pathogen_quarantine "{ $IP }"
