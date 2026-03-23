#!/usr/bin/env bash
# script: nft_list_rules.sh
# Layer: Infrastructure (OS boundary)
# Purpose: Outputs the current nftables ruleset in JSON format.

nft -j list ruleset
