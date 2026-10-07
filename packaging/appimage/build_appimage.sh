#!/usr/bin/env bash
# ==============================================================================
# Script de Construção do Pacote Universal AppImage
# ==============================================================================
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"
APP_DIR="${DIST_DIR}/AppDir"
OUTPUT_APPIMAGE="${DIST_DIR}/Photosheet-1.4.5-x86_64.AppImage"

echo "==> Compilando Photosheet em modo Release..."
cd "${ROOT_DIR}"
cargo build --release

echo "==> Estruturando AppDir em ${APP_DIR}..."
rm -rf "${APP_DIR}"
mkdir -p "${APP_DIR}/usr/bin"
mkdir -p "${APP_DIR}/usr/share/applications"
mkdir -p "${APP_DIR}/usr/share/icons/hicolor"

# Copiar binário
cp -f "${ROOT_DIR}/target/release/photosheet" "${APP_DIR}/usr/bin/photosheet"
chmod 755 "${APP_DIR}/usr/bin/photosheet"

# Copiar AppRun
cp -f "${ROOT_DIR}/packaging/appimage/AppRun" "${APP_DIR}/AppRun"
chmod 755 "${APP_DIR}/AppRun"

# Copiar desktop file e ícones
cp -f "${ROOT_DIR}/data/org.photosheet.Photosheet.desktop" "${APP_DIR}/usr/share/applications/"
cp -f "${ROOT_DIR}/data/org.photosheet.Photosheet.desktop" "${APP_DIR}/"

cp -rf "${ROOT_DIR}/data/icons/hicolor/"* "${APP_DIR}/usr/share/icons/hicolor/"
cp -f "${ROOT_DIR}/data/icons/hicolor/256x256/apps/photosheet.png" "${APP_DIR}/photosheet.png"
cp -f "${APP_DIR}/photosheet.png" "${APP_DIR}/.DirIcon"

echo "==> AppDir montado com sucesso."

# Empacotamento com appimagetool (se presente)
if command -v appimagetool >/dev/null 2>&1; then
    echo "==> Executando appimagetool..."
    ARCH=x86_64 appimagetool "${APP_DIR}" "${OUTPUT_APPIMAGE}"
    chmod +x "${OUTPUT_APPIMAGE}"
    echo "==> AppImage gerado com sucesso: ${OUTPUT_APPIMAGE}"
else
    echo "Nota: appimagetool não foi encontrado no PATH do sistema."
    echo "A estrutura AppDir está pronta em ${APP_DIR}."
    echo "Para gerar o arquivo .AppImage manualmente:"
    echo "  ARCH=x86_64 ./appimagetool-x86_64.AppImage ${APP_DIR} ${OUTPUT_APPIMAGE}"
fi
