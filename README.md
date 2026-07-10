<div align="center">

<img src="assets/clippy.png" alt="Clippy Logo" width="100"/>

# Clippy

**A native clipboard manager for GNOME Linux**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Built with Rust](https://img.shields.io/badge/Built_with-Rust-orange?logo=rust)](https://www.rust-lang.org/)
[![GTK4](https://img.shields.io/badge/GTK-4-4A90D9?logo=gnome)](https://gtk.org/)
[![libadwaita](https://img.shields.io/badge/libadwaita-1.x-5C2D91)](https://gnome.pages.gitlab.gnome.org/libadwaita/)

</div>

Clippy is a lightweight clipboard history manager built natively in Rust using GTK4 and libadwaita. It follows the GNOME Human Interface Guidelines, stores everything locally, and runs quietly in the background.

---

## Features

| Feature | Description |
|---|---|
| Clipboard history | Captures text and images as you copy, stored locally in SQLite |
| Live search | Filters clipboard history in real time as you type |
| Pin items | Pinned entries survive "Clear All" |
| Image support | Captures and previews images alongside text entries |
| Global hotkey | Configurable system-wide shortcut to toggle the window (default: `Super+V`) |
| Drag and drop | Drag a clipboard entry directly into another app |
| Deletion with undo panel | Slide-out confirmation before removing an entry |
| Always on top | Optional window pin to keep Clippy above other windows |
| Theme aware | Follows your GNOME light/dark color scheme automatically |
| Settings panel | Configure history limit, hotkey, and autostart from within the app |
| Autostart | Optionally starts with your GNOME session |
| Local only | No cloud, no sync, no telemetry |

---

## Screenshots

*(To be added)*

---

## Tech Stack

| Layer | Technology |
|---|---|
| Language | Rust |
| UI toolkit | GTK4 via gtk4-rs |
| Design system | libadwaita |
| Database | SQLite via rusqlite |
| Clipboard access | arboard |
| Image handling | image |
| Hashing | sha2 |
| Timestamps | chrono |
| Global shortcut | GNOME custom-keybinding gsettings schema |

---

## Installation

### Prerequisites

Ubuntu / Debian:

```bash
sudo apt install libgtk-4-dev libadwaita-1-dev pkg-config build-essential
```

Fedora:

```bash
sudo dnf install gtk4-devel libadwaita-devel pkg-config
```

### Install

```bash
git clone https://github.com/CharanMunur/Clippy.git
cd Clippy
./install.sh
```

This builds a release binary, installs it to `~/.local/bin`, and registers the app icon and `.desktop` launcher. To skip autostart:

```bash
./install.sh --no-autostart
```

### Uninstall

```bash
./uninstall.sh
```

---

## Running Locally (Development)

```bash
cargo run
```

Run as a background process with no window:

```bash
cargo run -- --background
```

Toggle the window from a terminal or custom shortcut:

```bash
cargo run -- --toggle
```

---

## Project Structure

```
clippy/
├── src/
│   ├── main.rs       # App entry point, window builder, CSS, menu actions
│   ├── ui.rs         # Row layout, clipboard operations, list rendering
│   ├── settings.rs   # Settings view, autostart toggle, hotkey config
│   ├── hotkey.rs     # GNOME gsettings custom keybinding writer
│   ├── poller.rs     # Background clipboard polling thread
│   └── db.rs         # SQLite schema, CRUD, config store
├── icons/            # Custom symbolic SVG icons
├── assets/           # App icon
├── install.sh
├── uninstall.sh
└── clippy.desktop
```

---

## Keyboard Shortcuts

| Action | Shortcut |
|---|---|
| Toggle Clippy window | `Super+V` (configurable in Settings) |
| Copy item | Left-click on an entry |
| Pin / unpin item | Click the pin icon on an entry |
| Open item actions | Click the menu icon on an entry |
| Delete item | Open actions, then click delete |

---

## Configuration

Configuration is stored in the local SQLite database at `~/.local/share/clippy/clippy.db`, under a `config` table. From the Settings panel inside the app you can change:

- Global hotkey
- History limit
- Autostart on login

---

## Contributing

Open an issue to discuss a change before starting significant work, or submit a pull request directly for small fixes.

1. Fork the repository
2. Create a feature branch: `git checkout -b feat/your-feature`
3. Commit your changes: `git commit -m "feat: add your feature"`
4. Push and open a pull request

---

## Author

Charan Munur
[charanmunur.in](https://www.charanmunur.in) · [GitHub](https://github.com/CharanMunur)

---

## License

Available under the [MIT License](LICENSE).