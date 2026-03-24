#!/bin/bash
cd /home/eehcx/source/Repos/nftables-tui

echo "=== Modo desarrollo de Pathogen ==="
echo "Este script prueba la aplicación en modo desarrollo (sin sudo)"
echo ""

# Establecer modo desarrollo
export PATHOGEN_DEV_MODE=1

# Limpiar reglas existentes (solo para limpieza)
sudo nft flush ruleset 2>/dev/null || true

echo "1. Compilando aplicación..."
cargo build --quiet

echo "2. Ejecutando aplicación por 5 segundos..."
echo "   (Presiona Ctrl+C para salir antes)"
echo ""
timeout 5 ./target/debug/pathogen 2>&1 | grep -v "^\[DEV\]"

echo ""
echo "=== Pruebas de funcionalidad ==="
echo ""
echo "3. Probando script de rate limit (simulado)..."
export PATHOGEN_DEV_MODE=1
./scripts/nft_rate_limit.sh 8080 tcp 10 minute

echo ""
echo "4. Probando script de cuarentena (simulado)..."
./scripts/nft_quarantine_ip.sh 192.168.1.100

echo ""
echo "=== Estado actual de nftables ==="
sudo nft list ruleset 2>/dev/null | head -20 || echo "(No hay reglas o error)"