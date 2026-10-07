#!/usr/bin/env bash
# ==============================================================================
# Script de Construção do Pacote Debian / Ubuntu (.deb)
# ==============================================================================
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PKG_NAME="photosheet"
VERSION="1.4.5"
ARCH="amd64"
DEB_BUILD_DIR="${ROOT_DIR}/dist/deb/${PKG_NAME}_${VERSION}_${ARCH}"

echo "==> Compilando Photosheet em modo Release..."
cd "${ROOT_DIR}"
cargo build --release

echo "==> Estruturando diretório de pacote .deb em ${DEB_BUILD_DIR}..."
rm -rf "${DEB_BUILD_DIR}"
mkdir -p "${DEB_BUILD_DIR}/DEBIAN"
mkdir -p "${DEB_BUILD_DIR}/usr/bin"
mkdir -p "${DEB_BUILD_DIR}/usr/share/applications"
mkdir -p "${DEB_BUILD_DIR}/usr/share/icons/hicolor"

# Arquivo de controle Debian
cat << EOF > "${DEB_BUILD_DIR}/DEBIAN/control"
Package: ${PKG_NAME}
Version: ${VERSION}
Section: graphics
Priority: optional
Architecture: ${ARCH}
Maintainer: Nilson Miranda <nilsonpmjr@gmail.com>
Depends: libgtk-4-1 (>= 4.12), libadwaita-1-0 (>= 1.4), libvulkan1, libc6
Description: Native Linux image editor with GPU Vulkan rendering
 Photosheet is a high-performance image editor and compositor designed
 natively for Linux with GTK4, Libadwaita, and Vulkan GPU acceleration.
EOF

# Instalar binário
cp -f "${ROOT_DIR}/target/release/photosheet" "${DEB_BUILD_DIR}/usr/bin/"
chmod 755 "${DEB_BUILD_DIR}/usr/bin/photosheet"

# Instalar arquivo .desktop
cp -f "${ROOT_DIR}/data/org.photosheet.Photosheet.desktop" "${DEB_BUILD_DIR}/usr/share/applications/"
chmod 644 "${DEB_BUILD_DIR}/usr/share/applications/org.photosheet.Photosheet.desktop"

# Instalar ícones
cp -rf "${ROOT_DIR}/data/icons/hicolor/"* "${DEB_BUILD_DIR}/usr/share/icons/hicolor/"

# Construir pacote com dpkg-deb se disponível, ou empacotar via ar/tar
OUTPUT_DEB="${ROOT_DIR}/dist/${PKG_NAME}_${VERSION}_${ARCH}.deb"
if command -v dpkg-deb >/dev/null 2>&1; then
    dpkg-deb --build --root-owner-group "${DEB_BUILD_DIR}" "${OUTPUT_DEB}"
    echo "==> Pacote .deb gerado com sucesso via dpkg-deb: ${OUTPUT_DEB}"
else
    echo "==> dpkg-deb não encontrado, empacotando .deb nativamente com ar e tar..."
    TMP_DEB_DIR="$(mktemp -d)"
    echo "2.0" > "${TMP_DEB_DIR}/debian-binary"
    tar --numeric-owner --group=0 --owner=0 -czf "${TMP_DEB_DIR}/control.tar.gz" -C "${DEB_BUILD_DIR}/DEBIAN" .
    tar --numeric-owner --group=0 --owner=0 -czf "${TMP_DEB_DIR}/data.tar.gz" -C "${DEB_BUILD_DIR}" usr
    (cd "${TMP_DEB_DIR}" && ar -rc "${OUTPUT_DEB}" debian-binary control.tar.gz data.tar.gz)
    rm -rf "${TMP_DEB_DIR}"
    echo "==> Pacote .deb gerado com sucesso: ${OUTPUT_DEB}"
fi

