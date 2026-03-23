#!/usr/bin/env bash
# script: nft_get_logs.sh
# Layer: Infrastructure
# Purpose: Retrieves the latest kernel logs related to our firewall drops.

# We grep for 'pathogen-drop' or generic nftables drops if configured
# Using journalctl to get the last 50 kernel messages matching pathogen
journalctl -k --no-pager -n 50 | grep "pathogen-drop" | tail -n 50 || true
