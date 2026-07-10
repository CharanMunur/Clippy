#!/usr/bin/env bash
set -e

# ─── Colors ───────────────────────────────────────────────────────────────────
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}▸ Uninstalling Clippy...${NC}"

# Stop running instance if any
pkill -x clippy 2>/dev/null && echo "  Stopped running Clippy instance." || true

# Remove binary
rm -f "$HOME/.local/bin/clippy"

# Remove app icon
rm -f "$HOME/.local/share/icons/hicolor/256x256/apps/clippy.png"
gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true

# Remove custom symbolic icons
rm -rf "$HOME/.local/share/clippy"

# Remove .desktop entry
rm -f "$HOME/.local/share/applications/clippy.desktop"
update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true

# Remove autostart
rm -f "$HOME/.config/autostart/clippy.desktop"

echo -e "${GREEN}✓ Clippy uninstalled successfully.${NC}"
