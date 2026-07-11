use std::process::Command;

/// Updates the GNOME global shortcut binding.
///
/// If `binding` is empty, the custom shortcut is disabled/removed.
/// Otherwise, it registers a custom media key under `/org/gnome/.../custom-keybindings/clippy/`.
pub fn update_gnome_shortcut(binding: &str) -> std::io::Result<()> {
    // 1. Get current custom keybindings list
    let output = Command::new("gsettings")
        .args(["get", "org.gnome.settings-daemon.plugins.media-keys", "custom-keybindings"])
        .output()?;
    let out_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let clippy_path = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/clippy/";

    // Parse the current array
    let mut paths = Vec::new();
    if out_str != "@as []" && !out_str.is_empty() {
        let clean = out_str.trim_start_matches('[').trim_end_matches(']');
        for part in clean.split(',') {
            let p = part.trim().trim_matches('\'').trim_matches('"');
            if !p.is_empty() {
                paths.push(p.to_string());
            }
        }
    }

    let has_clippy = paths.iter().any(|p| p == clippy_path);

    if binding.trim().is_empty() {
        // Remove from list if present
        if has_clippy {
            paths.retain(|p| p != clippy_path);
            let new_list = format!("[{}]", paths.iter().map(|p| format!("'{}'", p)).collect::<Vec<_>>().join(", "));
            let _ = Command::new("gsettings")
                .args(["set", "org.gnome.settings-daemon.plugins.media-keys", "custom-keybindings", &new_list])
                .status();
        }
    } else {
        // Add to list if not present
        if !has_clippy {
            paths.push(clippy_path.to_string());
            let new_list = format!("[{}]", paths.iter().map(|p| format!("'{}'", p)).collect::<Vec<_>>().join(", "));
            let _ = Command::new("gsettings")
                .args(["set", "org.gnome.settings-daemon.plugins.media-keys", "custom-keybindings", &new_list])
                .status();
        }

        // Apply custom path properties
        let path_arg = format!("org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:{}", clippy_path);
        let _ = Command::new("gsettings").args(["set", &path_arg, "name", "Clippy Toggle"]).status();
        
        let current_exe = std::env::current_exe()
            .unwrap_or_else(|_| std::path::PathBuf::from("/usr/bin/clippy"));
        let exec_path = format!("{} --toggle", current_exe.to_string_lossy());
        let _ = Command::new("gsettings").args(["set", &path_arg, "command", &exec_path]).status();
        let _ = Command::new("gsettings").args(["set", &path_arg, "binding", binding]).status();
    }

    Ok(())
}
