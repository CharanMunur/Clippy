# Clippy

An open-source clipboard manager for Ubuntu/Linux, built natively using Rust, GTK4, and libadwaita. It integrates seamlessly with the GNOME desktop environment, providing a native look and feel.

## Platform

Linux only, specifically GNOME-based desktops (Ubuntu, Fedora Workstation, Pop!_OS, etc.) where libadwaita theming applies.

## Current Status

The application is currently a skeleton window matching GNOME/Ubuntu styling with a native header bar.

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
