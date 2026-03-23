#!/usr/bin/env bash
# script: nft_unblock_port.sh
# Layer: Infrastructure (OS boundary)
# Purpose: Removes a block rule based on port and protocol.
# Usage: ./nft_unblock_port.sh <protocol> <port>

PROTOCOL=$1
PORT=$2

if [ -z "$PROTOCOL" ] || [ -z "$PORT" ]; then
    echo "Usage: $0 <protocol> <port>"
    exit 1
fi

HANDLE=$(nft -a list table inet filter | grep "tui-blocked-$PROTOCOL-$PORT" | grep -o 'handle [0-9]*' | awk '{print $2}')

if [ -n "$HANDLE" ]; then
    nft delete rule inet filter input handle $HANDLE
    echo "Rule deleted."
else
    echo "Rule not found."
    exit 1
fi
