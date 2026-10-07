#!/usr/bin/env bash
# ==============================================================================
# Script de Desinstalação do Photosheet (~/.local)
# ==============================================================================
set -euo pipefail

BIN_FILE="${HOME}/.local/bin/photosheet"
DESKTOP_FILE="${HOME}/.local/share/applications/org.photosheet.Photosheet.desktop"
ICONS_DIR="${HOME}/.local/share/icons/hicolor"

echo "==> Desinstalando Photosheet do diretório de usuário..."

if [ -f "${BIN_FILE}" ]; then
    rm -f "${BIN_FILE}"
    echo "  -> Removido: ${BIN_FILE}"
fi

if [ -f "${DESKTOP_FILE}" ]; then
    rm -f "${DESKTOP_FILE}"
    echo "  -> Removido: ${DESKTOP_FILE}"
fi

for size in 16x16 32x32 64x64 128x128 256x256 512x512; do
    icon="${ICONS_DIR}/${size}/apps/photosheet.png"
    if [ -f "${icon}" ]; then
        rm -f "${icon}"
        echo "  -> Removido: ${icon}"
    fi
done

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "${HOME}/.local/share/applications" || true
fi

echo "==> Desinstalação concluída com sucesso."
