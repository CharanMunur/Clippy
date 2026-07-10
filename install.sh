#!/usr/bin/env bash
set -e

# ─── Colors ───────────────────────────────────────────────────────────────────
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo -e "${BLUE}▸ Building Clippy (release)...${NC}"
cargo build --release --manifest-path "$SCRIPT_DIR/Cargo.toml"

# ─── Paths ────────────────────────────────────────────────────────────────────
INSTALL_BIN="$HOME/.local/bin/clippy"
INSTALL_ICON_DIR="$HOME/.local/share/icons/hicolor/256x256/apps"
INSTALL_DESKTOP_DIR="$HOME/.local/share/applications"
INSTALL_CUSTOM_ICONS_DIR="$HOME/.local/share/clippy/icons"
AUTOSTART_DIR="$HOME/.config/autostart"

# ─── Arguments ────────────────────────────────────────────────────────────────
AUTOSTART=true
for arg in "$@"; do
    if [ "$arg" = "--no-autostart" ]; then
        AUTOSTART=false
    fi
done

# ─── Binary ───────────────────────────────────────────────────────────────────
echo -e "${BLUE}▸ Installing binary to ~/.local/bin/...${NC}"
mkdir -p "$HOME/.local/bin"
cp "$SCRIPT_DIR/target/release/clippy" "$INSTALL_BIN"
chmod +x "$INSTALL_BIN"

# ─── App Icon ─────────────────────────────────────────────────────────────────
echo -e "${BLUE}▸ Installing app icon...${NC}"
mkdir -p "$INSTALL_ICON_DIR"
cp "$SCRIPT_DIR/assets/clippy.png" "$INSTALL_ICON_DIR/clippy.png"
gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true

# ─── Custom SVG Icons (pin buttons) ───────────────────────────────────────────
echo -e "${BLUE}▸ Installing custom symbolic icons...${NC}"
mkdir -p "$INSTALL_CUSTOM_ICONS_DIR/hicolor/scalable/actions"
cp -r "$SCRIPT_DIR/icons/." "$INSTALL_CUSTOM_ICONS_DIR/"

# ─── .desktop file ────────────────────────────────────────────────────────────
echo -e "${BLUE}▸ Installing .desktop entry...${NC}"
mkdir -p "$INSTALL_DESKTOP_DIR"
# Write a final .desktop with the resolved binary path
cat > "$INSTALL_DESKTOP_DIR/clippy.desktop" <<EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=Clippy
Comment=A native clipboard manager for GNOME/Linux
Exec=$INSTALL_BIN
Icon=clippy
Terminal=false
Categories=Utility;GTK;
Keywords=clipboard;copy;paste;history;
StartupNotify=false
StartupWMClass=org.gnome.Clippy
EOF
chmod +x "$INSTALL_DESKTOP_DIR/clippy.desktop"
update-desktop-database "$INSTALL_DESKTOP_DIR" 2>/dev/null || true

# ─── Autostart ────────────────────────────────────────────────────────────────
if [ "$AUTOSTART" = true ]; then
    echo -e "${BLUE}▸ Setting up autostart on login...${NC}"
    mkdir -p "$AUTOSTART_DIR"
    cat > "$AUTOSTART_DIR/clippy.desktop" <<EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=Clippy
Comment=A native clipboard manager for GNOME/Linux
Exec=$INSTALL_BIN
Icon=clippy
Terminal=false
Hidden=false
NoDisplay=false
X-GNOME-Autostart-enabled=true
StartupWMClass=org.gnome.Clippy
EOF
else
    echo -e "${BLUE}▸ Skipping autostart setup (--no-autostart)...${NC}"
fi

echo -e "${GREEN}✓ Clippy installed successfully!${NC}"
echo ""
echo "  Binary   → $INSTALL_BIN"
echo "  Icon     → $INSTALL_ICON_DIR/clippy.png"
echo "  Launcher → $INSTALL_DESKTOP_DIR/clippy.desktop"
if [ "$AUTOSTART" = true ]; then
    echo "  Autostart→ $AUTOSTART_DIR/clippy.desktop"
fi
echo ""
echo -e "${BLUE}  Make sure ~/.local/bin is in your PATH.${NC}"
echo "  Run 'clippy' or find it in your app launcher."
