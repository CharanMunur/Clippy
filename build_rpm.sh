#!/usr/bin/env bash
set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check if rpmbuild is installed
if ! command -v rpmbuild &> /dev/null; then
    echo -e "${RED}Error: rpmbuild is not installed.${NC}"
    echo "Please install it with: sudo apt install rpm"
    exit 1
fi

echo -e "${BLUE}▸ Building Clippy (release)...${NC}"
cargo build --release

# Setup Packaging Dir
VERSION="0.1.0"
RPM_DIR="target/rpm"
rm -rf "$RPM_DIR"
mkdir -p "$RPM_DIR/SPECS"
mkdir -p "$RPM_DIR/SOURCES"
mkdir -p "$RPM_DIR/BUILD"
mkdir -p "$RPM_DIR/RPMS"
mkdir -p "$RPM_DIR/SRPMS"

# Write RPM Spec File
echo -e "${BLUE}▸ Generating RPM spec file...${NC}"
cat > "$RPM_DIR/SPECS/clippy.spec" <<EOF
Name:           clippy
Version:        ${VERSION}
Release:        1%{?dist}
Summary:        A native clipboard manager for GNOME/Linux
License:        MIT
URL:            https://github.com/CharanMunur/Clippy
Requires:       wmctrl

# Disable debuginfo package creation since we are packaging a pre-compiled cargo binary
%define debug_package %{nil}

%description
A lightweight, privacy-first clipboard history manager built natively in Rust
using GTK4 and libadwaita. It integrates seamlessly into the GNOME desktop,
follows the GNOME Human Interface Guidelines, and supports text and images.

%install
mkdir -p %{buildroot}%{_bindir}
mkdir -p %{buildroot}%{_datadir}/applications
mkdir -p %{buildroot}%{_datadir}/metainfo
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/256x256/apps
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/scalable/actions

install -m 755 %{_sourcedir}/target/release/clippy %{buildroot}%{_bindir}/clippy
install -m 644 %{_sourcedir}/io.github.CharanMunur.Clippy.desktop %{buildroot}%{_datadir}/applications/io.github.CharanMunur.Clippy.desktop
install -m 644 %{_sourcedir}/io.github.CharanMunur.Clippy.metainfo.xml %{buildroot}%{_datadir}/metainfo/io.github.CharanMunur.Clippy.metainfo.xml
install -m 644 %{_sourcedir}/assets/clippy.png %{buildroot}%{_datadir}/icons/hicolor/256x256/apps/io.github.CharanMunur.Clippy.png
install -m 644 %{_sourcedir}/icons/hicolor/scalable/actions/clippy-pin-symbolic.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/actions/clippy-pin-symbolic.svg
install -m 644 %{_sourcedir}/icons/hicolor/scalable/actions/clippy-pin-active-symbolic.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/actions/clippy-pin-active-symbolic.svg

%files
%{_bindir}/clippy
%{_datadir}/applications/io.github.CharanMunur.Clippy.desktop
%{_datadir}/metainfo/io.github.CharanMunur.Clippy.metainfo.xml
%{_datadir}/icons/hicolor/256x256/apps/io.github.CharanMunur.Clippy.png
%{_datadir}/icons/hicolor/scalable/actions/clippy-pin-symbolic.svg
%{_datadir}/icons/hicolor/scalable/actions/clippy-pin-active-symbolic.svg
EOF

# Build RPM Package
echo -e "${BLUE}▸ Packaging into .rpm...${NC}"
rpmbuild -bb "$RPM_DIR/SPECS/clippy.spec" \
  --define "_topdir $PWD/$RPM_DIR" \
  --define "_sourcedir $PWD"

# Copy output RPM to target/rpm/
cp "$RPM_DIR/RPMS/x86_64/"*.rpm "$RPM_DIR/"

echo -e "${GREEN}✓ RPM package built successfully!${NC}"
echo "  Location: $(ls $RPM_DIR/*.rpm)"
