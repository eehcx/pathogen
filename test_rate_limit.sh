#!/bin/bash
cd /home/eehcx/source/Repos/nftables-tui
echo "Probando script de rate limit..."
echo "Parámetros: $1 $2 $3 $4"

# Primero limpiar reglas existentes
sudo nft flush ruleset 2>/dev/null || true

# Ejecutar el script
./scripts/nft_rate_limit.sh "$1" "$2" "$3" "$4"

# Verificar si se creó la regla
echo ""
echo "Reglas creadas:"
sudo nft list ruleset | grep -A2 -B2 "tui-ratelimit"