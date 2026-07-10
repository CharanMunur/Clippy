# Clippy

An open-source clipboard manager for Ubuntu/Linux, built natively using Rust, GTK4, and libadwaita. It integrates seamlessly with the GNOME desktop environment, providing a native look and feel.

## Platform

Linux only, specifically GNOME-based desktops (Ubuntu, Fedora Workstation, Pop!_OS, etc.) where libadwaita theming applies.

## Current Status

The application currently features:
- A clean native interface using Libadwaita, complete with a top `GtkSearchEntry` (live search bar), a sub-header bar with a bold title and "Clear all" button, a top-left window pin toggle button (always-on-top), and a non-resizable fixed window.
- Always-on-top window pinning using Mutter/X11 window manager hints triggered programmatically via `wmctrl` (targeting the active window `:ACTIVE:`).
- Live real-time search filtering querying the SQLite database dynamically as you type, complete with a **250ms debounce** to optimize performance during rapid typing.
- Individual floating cards showing content previews (limited to 1 line and 200 characters to prevent overflow, with a uniform **84px** height for text items and dynamic height for image items), with a tight **4px gap** between items, an integrated bottom-left timestamp, top-right "More actions" (`...`) menu button, and a bottom-right pushpin toggle (`clippy-pin-symbolic` / `clippy-pin-active-symbolic` overridden with custom angled outline/filled Bootstrap pushpin SVGs).
- Hover Interactivity: Hovering over any card turns the cursor into a pointer and shows a "Click to copy" tooltip. Left-clicking the card copies its content back to the clipboard instantly and triggers a native Libadwaita Toast notification showing what was copied.
- Toggleable delete-only action panel: Clicking `...` slides out only the Delete (bin) button from the right, with a `4px` gap between the card and the button, using custom CSS corner rounding resets to merge flush.
- Transparent ListBox and ListBoxRow backgrounds via custom GTK4 CSS, allowing cards to float natively on the window background.
- Clean symmetrical alignment of cards when the actions panel is hidden.
- Smooth slide-up animations for deleting list items using `GtkRevealer` and timed database updates.
- Native Drag & Drop support: drag any clipboard item card and drop it directly into other applications (transfers text content directly, and transfers images as file URI objects).
- An isolated SQLite database storage layer (`src/db.rs`) with CRUD operations and complete unit test coverage.
- A background clipboard polling thread (`src/poller.rs`) running every 400ms that captures new text and image data, hashes it, stores it in the database, and communicates updates back to the main thread via non-blocking channels.

## Getting Started

### Prerequisites

You must have the GTK4 and Libadwaita development libraries installed on your system. On Ubuntu, install them via apt:

```bash
sudo apt install libgtk-4-dev libadwaita-1-dev pkg-config build-essential
```

### Running the Application

To build and run the development version of the application:

```bash
cargo run
```

### Installation

To install Clippy permanently on your system (adds binary, desktop launcher icon, and autostart entry):

```bash
./install.sh
```

By default, Clippy is configured to autostart automatically on login. If you prefer to launch it manually, you can skip autostart setup by passing the `--no-autostart` flag:

```bash
./install.sh --no-autostart
```

To uninstall Clippy cleanly:

```bash
./uninstall.sh
```

## License

This project is open-source and available under the MIT License.
