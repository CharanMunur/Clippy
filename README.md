# Clippy

An open-source clipboard manager for Ubuntu/Linux, built natively using Rust, GTK4, and libadwaita. It integrates seamlessly with the GNOME desktop environment, providing a native look and feel.

## Platform

Linux only, specifically GNOME-based desktops (Ubuntu, Fedora Workstation, Pop!_OS, etc.) where libadwaita theming applies.

## Current Status

The application currently features:
- A Windows 11-inspired native interface using Libadwaita, complete with a top icon tab bar (with blue underline active indicators), a sub-header bar with a bold title and "Clear all" button, and a non-resizable fixed window.
- Individual floating cards showing content previews (limited to 3 lines and 200 characters to prevent overflow), with an integrated bottom-left timestamp, top-right "More actions" (`...`) menu button, and a bottom-right pushpin toggle.
- Toggleable external action panel containing elevated square copy-to-clipboard and delete-entry buttons that slide out smoothly next to the card using a `GtkRevealer` transition when the `...` menu is clicked.
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

## License

This project is open-source and available under the MIT License.
