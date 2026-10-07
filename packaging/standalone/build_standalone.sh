#!/usr/bin/env bash
# ==============================================================================
# Constrói o Pacote Portátil Standalone (.tar.gz e .zip) para o Usuário
# ==============================================================================
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"
PKG_NAME="photosheet-1.4.5-linux-x86_64"
TARGET_DIR="${DIST_DIR}/${PKG_NAME}"

echo "==> Compilando Photosheet em modo Release..."
cd "${ROOT_DIR}"
cargo build --release

echo "==> Montando estrutura do pacote standalone em ${TARGET_DIR}..."
rm -rf "${TARGET_DIR}"
mkdir -p "${TARGET_DIR}/icons"

# Copiar binário
cp -f "${ROOT_DIR}/target/release/photosheet" "${TARGET_DIR}/"
chmod +x "${TARGET_DIR}/photosheet"

# Copiar scripts de instalação
cp -f "${ROOT_DIR}/packaging/standalone/install.sh" "${TARGET_DIR}/"
cp -f "${ROOT_DIR}/packaging/standalone/uninstall.sh" "${TARGET_DIR}/"
chmod +x "${TARGET_DIR}/install.sh" "${TARGET_DIR}/uninstall.sh"

# Copiar desktop e ícones
cp -f "${ROOT_DIR}/data/org.photosheet.Photosheet.desktop" "${TARGET_DIR}/"
cp -rf "${ROOT_DIR}/data/icons/hicolor" "${TARGET_DIR}/icons/"

# Criar arquivo README de instruções
cat << 'EOF' > "${TARGET_DIR}/README.txt"
================================================================================
Photosheet 1.4.5 — Editor de Imagens Nativo para Linux
================================================================================

COMO INSTALAR (Sem necessidade de root/sudo):
1. Abra um terminal dentro desta pasta extraída.
2. Execute o script de instalação:
     ./install.sh
3. Pronto! O Photosheet estará disponível no menu de aplicativos do seu sistema
   e no caminho ~/.local/bin/photosheet.

COMO DESINSTALAR:
   Execute ./uninstall.sh a qualquer momento.
================================================================================
EOF

echo "==> Compactando arquivos .tar.gz e .zip em ${DIST_DIR}..."
cd "${DIST_DIR}"
tar -czf "${PKG_NAME}.tar.gz" "${PKG_NAME}"
zip -rq "${PKG_NAME}.zip" "${PKG_NAME}"

echo "==> Pacotes gerados com sucesso:"
ls -lh "${DIST_DIR}/${PKG_NAME}.tar.gz" "${DIST_DIR}/${PKG_NAME}.zip"
