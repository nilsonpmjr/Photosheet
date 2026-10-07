#!/usr/bin/env bash
# ==============================================================================
# Script de Instalação do Photosheet no Diretório do Usuário (~/.local)
# Sem necessidade de privilégios de root (sudo).
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="${HOME}/.local/bin"
SHARE_DIR="${HOME}/.local/share"
APPS_DIR="${SHARE_DIR}/applications"
ICONS_DIR="${SHARE_DIR}/icons"

echo "==> Instalando Photosheet para o usuário $(whoami)..."

# 1. Garantir que os diretórios existam
mkdir -p "${BIN_DIR}"
mkdir -p "${APPS_DIR}"
mkdir -p "${ICONS_DIR}"

# 2. Copiar binário executável
if [ -f "${SCRIPT_DIR}/photosheet" ]; then
    cp -f "${SCRIPT_DIR}/photosheet" "${BIN_DIR}/photosheet"
    chmod +x "${BIN_DIR}/photosheet"
    echo "  -> Binário instalado em: ${BIN_DIR}/photosheet"
else
    echo "Erro: Binário 'photosheet' não encontrado em ${SCRIPT_DIR}."
    exit 1
fi

# 3. Copiar ícones de alta resolução
if [ -d "${SCRIPT_DIR}/icons" ]; then
    cp -rf "${SCRIPT_DIR}/icons/"* "${ICONS_DIR}/"
    echo "  -> Ícones instalados em: ${ICONS_DIR}/"
fi

# 4. Copiar arquivo .desktop do menu de aplicativos
DESKTOP_SRC="${SCRIPT_DIR}/org.photosheet.Photosheet.desktop"
if [ -f "${DESKTOP_SRC}" ]; then
    # Ajusta o caminho absoluto do Exec para o caso do ~/.local/bin não estar no PATH global
    sed "s|Exec=photosheet|Exec=${BIN_DIR}/photosheet|g" "${DESKTOP_SRC}" > "${APPS_DIR}/org.photosheet.Photosheet.desktop"
    chmod +x "${APPS_DIR}/org.photosheet.Photosheet.desktop"
    echo "  -> Atalho de aplicativo instalado em: ${APPS_DIR}/org.photosheet.Photosheet.desktop"
fi

# 5. Atualizar bancos de dados do sistema (se os comandos estiverem disponíveis)
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "${APPS_DIR}" || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "${ICONS_DIR}/hicolor" || true
fi

echo ""
echo "==> Sucesso! O Photosheet foi instalado com sucesso."
if [[ ":$PATH:" != *":${BIN_DIR}:"* ]]; then
    echo "Nota: Adicione '${BIN_DIR}' ao seu PATH se ainda não o fez, por exemplo adicionando ao seu ~/.bashrc ou ~/.zshrc:"
    echo '  export PATH="$HOME/.local/bin:$PATH"'
fi
echo "Você já pode executar 'photosheet' ou abri-lo pelo menu de aplicativos do seu sistema!"
