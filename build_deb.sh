#!/usr/bin/env bash
set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}▸ Building Clippy (release)...${NC}"
cargo build --release

# Setup Packaging Dir
VERSION="0.1.0"
PKG_DIR="target/debian/clippy_${VERSION}_amd64"
rm -rf "target/debian"
mkdir -p "$PKG_DIR/DEBIAN"
mkdir -p "$PKG_DIR/usr/bin"
mkdir -p "$PKG_DIR/usr/share/applications"
mkdir -p "$PKG_DIR/usr/share/icons/hicolor/256x256/apps"
mkdir -p "$PKG_DIR/usr/share/icons/hicolor/scalable/actions"

# Copy Files
echo -e "${BLUE}▸ Copying files to package directory...${NC}"
cp "target/release/clippy" "$PKG_DIR/usr/bin/clippy"
chmod 755 "$PKG_DIR/usr/bin/clippy"

cp "io.github.CharanMunur.Clippy.desktop" "$PKG_DIR/usr/share/applications/io.github.CharanMunur.Clippy.desktop"
chmod 644 "$PKG_DIR/usr/share/applications/io.github.CharanMunur.Clippy.desktop"

cp "assets/clippy.png" "$PKG_DIR/usr/share/icons/hicolor/256x256/apps/io.github.CharanMunur.Clippy.png"
chmod 644 "$PKG_DIR/usr/share/icons/hicolor/256x256/apps/io.github.CharanMunur.Clippy.png"

cp "icons/hicolor/scalable/actions/clippy-pin-symbolic.svg" "$PKG_DIR/usr/share/icons/hicolor/scalable/actions/clippy-pin-symbolic.svg"
cp "icons/hicolor/scalable/actions/clippy-pin-active-symbolic.svg" "$PKG_DIR/usr/share/icons/hicolor/scalable/actions/clippy-pin-active-symbolic.svg"
chmod 644 "$PKG_DIR/usr/share/icons/hicolor/scalable/actions/"*.svg

# Write Debian Control File
echo -e "${BLUE}▸ Generating control file...${NC}"
cat > "$PKG_DIR/DEBIAN/control" <<EOF
Package: clippy
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: amd64
Maintainer: Charan Munur <https://www.charanmunur.in>
Depends: libgtk-4-1, libadwaita-1-0, libsqlite3-0, wmctrl
Description: A native clipboard manager for GNOME/Linux
 A lightweight, privacy-first clipboard history manager built natively in Rust
 using GTK4 and libadwaita. It integrates seamlessly into the GNOME desktop,
 follows the GNOME Human Interface Guidelines, and supports text and images.
EOF

# Build Deb Package
echo -e "${BLUE}▸ Packaging into .deb...${NC}"
dpkg-deb --build "$PKG_DIR"

echo -e "${GREEN}✓ Debian package built successfully!${NC}"
echo "  Location: target/debian/clippy_${VERSION}_amd64.deb"
