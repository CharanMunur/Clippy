use adw::prelude::*;
use gtk::{Box, Orientation, Label, ScrolledWindow, Switch, Entry, Button, Stack, ListBox, SearchEntry, MenuButton, EventControllerKey, gio};
use gtk::glib;
use adw::{Window as AdwWindow, HeaderBar as AdwHeaderBar};
use crate::db;
use crate::hotkey;

/// Converts GTK accelerator format to human-readable: `<Super>j` → `Super + J`
fn gtk_to_human(accel: &str) -> String {
    let mut s = accel.to_string();
    let mut parts: Vec<String> = Vec::new();
    // Extract modifiers
    for (tag, label) in &[("<Super>", "Super"), ("<Ctrl>", "Ctrl"), ("<Alt>", "Alt"), ("<Shift>", "Shift")] {
        if s.contains(tag) {
            parts.push(label.to_string());
            s = s.replace(tag, "");
        }
    }
    // Remaining is the key
    if !s.is_empty() {
        parts.push(s.to_uppercase());
    }
    parts.join(" + ")
}


/// Enables or disables autostart on login by creating/removing the desktop entry in ~/.config/autostart/
pub fn set_autostart_enabled(enabled: bool) -> std::io::Result<()> {
    if let Some(bd) = directories::BaseDirs::new() {
        let mut autostart_dir = bd.config_dir().to_path_buf();
        autostart_dir.push("autostart");
        std::fs::create_dir_all(&autostart_dir)?;
        
        let autostart_file = autostart_dir.join("clippy.desktop");
        
        if enabled {
            let install_bin = format!("{}/.local/bin/clippy", std::env::var("HOME").unwrap_or_default());
            let desktop_content = format!(
                "[Desktop Entry]\n\
                 Version=1.0\n\
                 Type=Application\n\
                 Name=Clippy\n\
                 Comment=A native clipboard manager for GNOME/Linux\n\
                 Exec={} --background\n\
                 Icon=clippy\n\
                 Terminal=false\n\
                 Hidden=false\n\
                 NoDisplay=false\n\
                 X-GNOME-Autostart-enabled=true\n\
                 StartupWMClass=org.gnome.Clippy\n",
                install_bin
            );
            std::fs::write(&autostart_file, desktop_content)?;
        } else {
            if autostart_file.exists() {
                std::fs::remove_file(autostart_file)?;
            }
        }
    }
    Ok(())
}

/// Builds the Settings view ScrolledWindow and sets up all action handlers.
pub fn build_settings_view(
    app: &adw::Application,
    stack: &Stack,
    list_box: &ListBox,
    search_entry: &SearchEntry,
    menu_button: &MenuButton,
    pin_win_btn: &Button,
    back_btn: &Button,
    header_title: &Label,
    refresh_list_fn: impl Fn(&ListBox, &Stack, &SearchEntry) + 'static + Clone,
) -> ScrolledWindow {
    let settings_scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .vexpand(true)
        .build();

    let settings_box = Box::new(Orientation::Vertical, 0);
    settings_box.set_margin_top(16);
    settings_box.set_margin_bottom(16);
    settings_box.set_margin_start(16);
    settings_box.set_margin_end(16);

    // 1. Autostart Setting Card
    let autostart_card = Box::new(Orientation::Horizontal, 12);
    autostart_card.add_css_class("settings-card");
    
    let autostart_text_box = Box::new(Orientation::Vertical, 4);
    let autostart_title = Label::builder()
        .label("Autostart on Login")
        .halign(gtk::Align::Start)
        .build();
    autostart_title.add_css_class("settings-label-title");
    let autostart_sub = Label::builder()
        .label("Start Clippy automatically when you log in")
        .halign(gtk::Align::Start)
        .build();
    autostart_sub.add_css_class("settings-label-subtitle");
    autostart_text_box.append(&autostart_title);
    autostart_text_box.append(&autostart_sub);
    
    let autostart_switch = Switch::builder()
        .valign(gtk::Align::Center)
        .build();
    
    let autostart_spacer = Box::new(Orientation::Horizontal, 0);
    autostart_spacer.set_hexpand(true);
    
    autostart_card.append(&autostart_text_box);
    autostart_card.append(&autostart_spacer);
    autostart_card.append(&autostart_switch);
    settings_box.append(&autostart_card);

    // 2. Shortcut Setting Card
    let shortcut_card = Box::new(Orientation::Horizontal, 12);
    shortcut_card.add_css_class("settings-card");

    let shortcut_text_box = Box::new(Orientation::Vertical, 4);
    let shortcut_title = Label::builder()
        .label("Global Shortcut")
        .halign(gtk::Align::Start)
        .build();
    shortcut_title.add_css_class("settings-label-title");
    let shortcut_sub = Label::builder()
        .label("Click to record a new key combination")
        .halign(gtk::Align::Start)
        .build();
    shortcut_sub.add_css_class("settings-label-subtitle");
    shortcut_text_box.append(&shortcut_title);
    shortcut_text_box.append(&shortcut_sub);

    // Backing store — always holds the GTK accel string e.g. "<Super>j"
    let shortcut_recorded = std::rc::Rc::new(std::cell::RefCell::new(String::from("<Super>j")));

    // Button that shows the human-readable current shortcut
    let shortcut_btn = Button::builder()
        .label("Super + J")
        .valign(gtk::Align::Center)
        .halign(gtk::Align::End)
        .build();
    shortcut_btn.add_css_class("flat");

    let shortcut_spacer = Box::new(Orientation::Horizontal, 0);
    shortcut_spacer.set_hexpand(true);

    shortcut_card.append(&shortcut_text_box);
    shortcut_card.append(&shortcut_spacer);
    shortcut_card.append(&shortcut_btn);
    settings_box.append(&shortcut_card);

    // Recorder dialog on button click
    let shortcut_recorded_btn = shortcut_recorded.clone();
    let shortcut_btn_clone = shortcut_btn.clone();
    shortcut_btn.connect_clicked(move |_| {
        // Get parent from thread-local — always reliable even on Wayland
        let parent = crate::WINDOW.with(|w| w.borrow().clone());

        // Use adw::Window so the compositor positions it over the parent
        let dialog = AdwWindow::builder()
            .modal(true)
            .resizable(false)
            .default_width(360)
            .destroy_with_parent(true)
            .build();
        if let Some(p) = parent.as_ref() {
            dialog.set_transient_for(Some(p));
        }

        // Content box inside a window-level Box with a HeaderBar
        let outer = Box::new(Orientation::Vertical, 0);

        // HeaderBar with Cancel (start) and Set (end)
        let hbar = AdwHeaderBar::new();
        hbar.set_show_end_title_buttons(false);
        hbar.set_show_start_title_buttons(false);

        let cancel_btn = Button::builder().label("Cancel").build();
        cancel_btn.add_css_class("flat");

        let set_btn = Button::builder().label("Set").build();
        set_btn.add_css_class("suggested-action");
        set_btn.set_sensitive(false);

        hbar.pack_start(&cancel_btn);
        hbar.pack_end(&set_btn);
        outer.append(&hbar);

        // Body
        let vbox = Box::new(Orientation::Vertical, 12);
        vbox.set_margin_top(20);
        vbox.set_margin_bottom(28);
        vbox.set_margin_start(24);
        vbox.set_margin_end(24);
        vbox.set_valign(gtk::Align::Center);
        vbox.set_vexpand(true);

        let title_lbl = Label::builder().label("Press a key combination").build();
        title_lbl.add_css_class("title-4");

        let hint_lbl = Label::builder()
            .label("Requires Super, Ctrl, or Alt as a modifier")
            .build();
        hint_lbl.add_css_class("dim-label");
        hint_lbl.add_css_class("caption");

        let recording_lbl = Label::builder().label("Waiting for keys…").build();
        recording_lbl.add_css_class("title-2");
        recording_lbl.set_margin_top(20);
        recording_lbl.set_margin_bottom(20);

        vbox.append(&title_lbl);
        vbox.append(&hint_lbl);
        vbox.append(&recording_lbl);
        outer.append(&vbox);
        dialog.set_content(Some(&outer));

        // Captured accel shared between key controller and Set button
        let captured: std::rc::Rc<std::cell::RefCell<Option<String>>> =
            std::rc::Rc::new(std::cell::RefCell::new(None));

        let captured_key = captured.clone();
        let recording_lbl_key = recording_lbl.clone();
        let set_btn_key = set_btn.clone();
        let dialog_key = dialog.clone();

        let key_ctrl = EventControllerKey::new();
        key_ctrl.connect_key_pressed(move |_, keyval, _code, state| {
            use gtk::gdk;
            // Escape = cancel
            if keyval == gdk::Key::Escape {
                dialog_key.close();
                return glib::Propagation::Stop;
            }
            // Skip bare modifier presses
            if matches!(keyval,
                gdk::Key::Super_L | gdk::Key::Super_R |
                gdk::Key::Control_L | gdk::Key::Control_R |
                gdk::Key::Alt_L | gdk::Key::Alt_R |
                gdk::Key::Shift_L | gdk::Key::Shift_R |
                gdk::Key::Hyper_L | gdk::Key::Hyper_R |
                gdk::Key::Meta_L | gdk::Key::Meta_R
            ) { return glib::Propagation::Proceed; }

            // Require at least one non-shift modifier
            let has_mod = state.intersects(
                gdk::ModifierType::SUPER_MASK |
                gdk::ModifierType::CONTROL_MASK |
                gdk::ModifierType::ALT_MASK,
            );
            if !has_mod {
                recording_lbl_key.set_label("Add Super, Ctrl, or Alt");
                return glib::Propagation::Stop;
            }

            // Build GTK accel string
            let mut accel = String::new();
            if state.contains(gdk::ModifierType::SUPER_MASK)   { accel.push_str("<Super>"); }
            if state.contains(gdk::ModifierType::CONTROL_MASK) { accel.push_str("<Ctrl>");  }
            if state.contains(gdk::ModifierType::ALT_MASK)     { accel.push_str("<Alt>");   }
            if state.contains(gdk::ModifierType::SHIFT_MASK)   { accel.push_str("<Shift>"); }
            if let Some(c) = keyval.to_unicode() {
                accel.push(c.to_lowercase().next().unwrap_or(c));
            } else if let Some(n) = keyval.name() {
                accel.push_str(n.as_str());
            }

            *captured_key.borrow_mut() = Some(accel.clone());
            recording_lbl_key.set_label(&gtk_to_human(&accel));
            set_btn_key.set_sensitive(true);
            glib::Propagation::Stop
        });
        dialog.add_controller(key_ctrl);

        // Cancel
        let dialog_cancel = dialog.clone();
        cancel_btn.connect_clicked(move |_| { dialog_cancel.close(); });

        // Set
        let shortcut_recorded_set = shortcut_recorded_btn.clone();
        let shortcut_btn_lbl = shortcut_btn_clone.clone();
        let dialog_set = dialog.clone();
        set_btn.connect_clicked(move |_| {
            if let Some(accel) = captured.borrow().clone() {
                *shortcut_recorded_set.borrow_mut() = accel.clone();
                shortcut_btn_lbl.set_label(&gtk_to_human(&accel));
            }
            dialog_set.close();
        });

        dialog.present();
    });

    // 3. History Limit Setting Card
    let limit_card = Box::new(Orientation::Horizontal, 12);
    limit_card.add_css_class("settings-card");
    
    let limit_text_box = Box::new(Orientation::Vertical, 4);
    let limit_title = Label::builder()
        .label("History Limit")
        .halign(gtk::Align::Start)
        .build();
    limit_title.add_css_class("settings-label-title");
    let limit_sub = Label::builder()
        .label("Maximum number of items to keep in history")
        .halign(gtk::Align::Start)
        .build();
    limit_sub.add_css_class("settings-label-subtitle");
    limit_text_box.append(&limit_title);
    limit_text_box.append(&limit_sub);
    
    let limit_entry = Entry::builder()
        .valign(gtk::Align::Center)
        .width_request(80)
        .halign(gtk::Align::End)
        .build();
    
    let limit_spacer = Box::new(Orientation::Horizontal, 0);
    limit_spacer.set_hexpand(true);
    
    limit_card.append(&limit_text_box);
    limit_card.append(&limit_spacer);
    limit_card.append(&limit_entry);
    settings_box.append(&limit_card);

    // 4. Save Button
    let save_btn = Button::builder()
        .label("Save Settings")
        .margin_top(16)
        .build();
    save_btn.add_css_class("clear-all-btn");
    settings_box.append(&save_btn);

    settings_scrolled.set_child(Some(&settings_box));

    // Register app-level preferences action to switch to the settings view
    let pref_action = gio::SimpleAction::new("preferences", None);
    
    let stack_clone = stack.clone();
    let menu_button_clone = menu_button.clone();
    let pin_win_btn_clone = pin_win_btn.clone();
    let back_btn_clone = back_btn.clone();
    let header_title_clone = header_title.clone();
    
    let autostart_switch_clone = autostart_switch.clone();
    let shortcut_recorded_pref = shortcut_recorded.clone();
    let shortcut_btn_pref = shortcut_btn.clone();
    let limit_entry_clone = limit_entry.clone();

    pref_action.connect_activate(move |_, _| {
        if let Ok(conn) = db::init_db() {
            let autostart_val = db::get_config_val(&conn, "autostart")
                .unwrap_or(None)
                .unwrap_or_else(|| "true".to_string());
            let shortcut_val = db::get_config_val(&conn, "shortcut")
                .unwrap_or(None)
                .unwrap_or_else(|| "<Super>j".to_string());
            let limit_val = db::get_config_val(&conn, "history_limit")
                .unwrap_or(None)
                .unwrap_or_else(|| "200".to_string());

            autostart_switch_clone.set_active(autostart_val == "true");
            // Sync the backing store and update the button label
            *shortcut_recorded_pref.borrow_mut() = shortcut_val.clone();
            shortcut_btn_pref.set_label(&gtk_to_human(&shortcut_val));
            limit_entry_clone.set_text(&limit_val);
        }

        stack_clone.set_visible_child_name("settings");
        menu_button_clone.set_visible(false);
        pin_win_btn_clone.set_visible(false);
        back_btn_clone.set_visible(true);
        header_title_clone.set_label("Settings");
    });
    app.add_action(&pref_action);

    // Connect Back button click (Cancel settings changes)
    let stack_clone_back = stack.clone();
    let menu_button_clone_back = menu_button.clone();
    let pin_win_btn_clone_back = pin_win_btn.clone();
    let back_btn_clone_back = back_btn.clone();
    let header_title_clone_back = header_title.clone();
    let list_box_clone_back = list_box.clone();
    let search_entry_clone_back = search_entry.clone();
    let refresh_list_fn_back = refresh_list_fn.clone();

    back_btn.connect_clicked(move |_| {
        refresh_list_fn_back(&list_box_clone_back, &stack_clone_back, &search_entry_clone_back);
        menu_button_clone_back.set_visible(true);
        pin_win_btn_clone_back.set_visible(true);
        back_btn_clone_back.set_visible(false);
        header_title_clone_back.set_label("Clippy");
    });

    // Connect Save button click
    let stack_clone_save = stack.clone();
    let menu_button_clone_save = menu_button.clone();
    let pin_win_btn_clone_save = pin_win_btn.clone();
    let back_btn_clone_save = back_btn.clone();
    let header_title_clone_save = header_title.clone();
    let list_box_clone_save = list_box.clone();
    let search_entry_clone_save = search_entry.clone();
    let refresh_list_fn_save = refresh_list_fn;

    save_btn.connect_clicked(move |_| {
        let autostart_enabled = autostart_switch.is_active();
        let autostart_str = if autostart_enabled { "true" } else { "false" };
        // Read the GTK accel string directly from the backing store
        let shortcut_gtk = shortcut_recorded.borrow().clone();
        let limit_str = limit_entry.text().to_string();

        if let Ok(conn) = db::init_db() {
            let _ = db::set_config_val(&conn, "autostart", autostart_str);
            let _ = db::set_config_val(&conn, "shortcut", &shortcut_gtk);
            let _ = db::set_config_val(&conn, "history_limit", &limit_str);

            let _ = hotkey::update_gnome_shortcut(&shortcut_gtk);
            let _ = set_autostart_enabled(autostart_enabled);

            if let Ok(limit) = limit_str.trim().parse::<usize>() {
                let _ = db::prune_entries(&conn, limit);
            }
        }

        refresh_list_fn_save(&list_box_clone_save, &stack_clone_save, &search_entry_clone_save);
        menu_button_clone_save.set_visible(true);
        pin_win_btn_clone_save.set_visible(true);
        back_btn_clone_save.set_visible(false);
        header_title_clone_save.set_label("Clippy");
    });

    settings_scrolled
}
