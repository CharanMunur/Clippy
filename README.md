<div align="center">

<img src="assets/clippy.png" alt="Clippy Logo" width="100"/>

# Clippy

**A native clipboard manager for GNOME Linux**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Built with Rust](https://img.shields.io/badge/Built_with-Rust-orange?logo=rust)](https://www.rust-lang.org/)
[![GTK4](https://img.shields.io/badge/GTK-4-4A90D9?logo=gnome)](https://gtk.org/)
[![libadwaita](https://img.shields.io/badge/libadwaita-1.x-5C2D91)](https://gnome.pages.gitlab.gnome.org/libadwaita/)
[![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20GNOME-success?logo=linux)](https://www.gnome.org/)

Clippy is a lightweight, privacy-first clipboard history manager built natively in Rust using GTK4 and libadwaita. It integrates seamlessly into the GNOME desktop, follows the Human Interface Guidelines, and runs quietly in the background — always ready when you need it.

</div>

---

## ✨ Features

| Feature | Description |
|---|---|
| 📋 **Clipboard History** | Automatically captures text and images as you copy, stored locally in SQLite |
| 🔍 **Live Search** | Instantly filter your clipboard history with a debounced real-time search bar |
| 📌 **Pin Items** | Pin important clipboard entries so they survive "Clear All" |
| 🖼️ **Image Support** | Captures and previews images alongside text entries |
| 🌐 **Global Hotkey** | Configurable system-wide keyboard shortcut to toggle the window (default: `Super+V`) |
| 🖱️ **Drag & Drop** | Drag any clipboard card and drop it directly into other apps |
| 🗑️ **Smooth Deletions** | Slide-out action panel with animated delete using `GtkRevealer` |
| 🔄 **Always on Top** | Optional window pin to keep Clippy above all other windows |
| 🌗 **Dark & Light Theme** | Fully theme-aware — adapts automatically to your GNOME color scheme |
| ⚙️ **Settings Panel** | Configure history limit, global hotkey, and autostart from within the app |
| 🚀 **Autostart** | Optionally starts with your GNOME session via a `.desktop` autostart entry |
| 🔒 **Privacy-first** | Everything stays local — no cloud, no sync, no telemetry |

---

## 📸 Screenshots

> Clippy adapts to both dark and light GNOME themes automatically.

---

## 🛠️ Tech Stack

| Layer | Technology |
|---|---|
| Language | [Rust](https://www.rust-lang.org/) |
| UI Toolkit | [GTK4](https://gtk.org/) via [`gtk4-rs`](https://gtk-rs.org/) |
| Design System | [libadwaita](https://gnome.pages.gitlab.gnome.org/libadwaita/) |
| Database | [SQLite](https://www.sqlite.org/) via [`rusqlite`](https://github.com/rusqlite/rusqlite) |
| Clipboard | [`arboard`](https://github.com/1Password/arboard) |
| Image Handling | [`image`](https://github.com/image-rs/image) |
| Hashing | [`sha2`](https://github.com/RustCrypto/hashes) |
| Timestamps | [`chrono`](https://github.com/chronotope/chrono) |
| Global Shortcuts | GNOME GSettings (`org.gnome.settings-daemon`) |

---

## 📦 Installation

### Prerequisites

Ensure you have the GTK4 and libadwaita development libraries installed. On **Ubuntu / Debian**:

```bash
sudo apt install libgtk-4-dev libadwaita-1-dev pkg-config build-essential
```

On **Fedora**:

```bash
sudo dnf install gtk4-devel libadwaita-devel pkg-config
```

### Install (Recommended)

Clone the repo and run the installer. This compiles a release build, installs the binary to `~/.local/bin`, registers the app icon, `.desktop` launcher, and optionally sets up autostart.

```bash
git clone https://github.com/CharanMunur/Clippy.git
cd Clippy
./install.sh
```

To skip autostart on login:

```bash
./install.sh --no-autostart
```

### Uninstall

```bash
./uninstall.sh
```

This removes the binary, launcher, icon, and autostart entry cleanly.

---

## 🚀 Running Locally (Development)

```bash
cargo run
```

To run in background mode (daemon, no window):

```bash
cargo run -- --background
```

To toggle the window from a terminal or custom shortcut:

```bash
cargo run -- --toggle
```

---

## 🗂️ Project Structure

```
clippy/
├── src/
│   ├── main.rs       # App entry point, window builder, CSS, menu actions
│   ├── ui.rs         # Card layout, clipboard operations, list rendering
│   ├── settings.rs   # Settings view, autostart toggle, hotkey config
│   ├── hotkey.rs     # GNOME GSettings custom keybinding writer
│   ├── poller.rs     # Background clipboard polling thread
│   └── db.rs         # SQLite schema, CRUD, config store
├── icons/            # Custom symbolic SVG icons
├── assets/           # App icon
├── install.sh        # One-command installer
├── uninstall.sh      # Clean uninstaller
└── clippy.desktop    # Desktop entry template
```

---

## ⌨️ Keyboard Shortcuts

| Action | Shortcut |
|---|---|
| Toggle Clippy window | `Super + V` *(configurable in Settings)* |
| Copy item | Left-click on any card |
| Pin / unpin item | Click the pin icon on a card |
| Open item actions | Click `···` on a card |
| Delete item | Open actions → click the bin icon |

---

## ⚙️ Configuration

All configuration is stored in a local SQLite database (`~/.local/share/clippy/clippy.db`) under a `config` table. You can change the following from the **Settings** panel inside the app:

- **Global Hotkey** — the system-wide shortcut to show/hide Clippy
- **History Limit** — maximum number of clipboard entries to retain
- **Autostart** — toggle whether Clippy starts with your GNOME session

---

## 🤝 Contributing

Contributions are welcome! Please open an issue first to discuss what you'd like to change, or submit a pull request directly for small fixes.

1. Fork the repository
2. Create a feature branch: `git checkout -b feat/your-feature`
3. Commit your changes: `git commit -m "feat: add your feature"`
4. Push and open a PR: `git push origin feat/your-feature`

---

## 👤 Author

**Charan Munur**
🌐 [charanmunur.in](https://www.charanmunur.in) · 🐙 [GitHub](https://github.com/CharanMunur)

---

## 📄 License

This project is open-source and available under the [MIT License](LICENSE).
