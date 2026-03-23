#!/usr/bin/env bash
# script: nft_unquarantine_ip.sh
IP=$1
if [ -z "$IP" ]; then exit 1; fi
nft delete element inet filter pathogen_quarantine "{ $IP }" || true