#!/usr/bin/env bash
# script: nft_list_quarantine.sh
nft -j list set inet filter pathogen_quarantine || echo '{"nftables":[]}'