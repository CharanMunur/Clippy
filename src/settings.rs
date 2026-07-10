use adw::prelude::*;
use gtk::{Box, Orientation, Label, ScrolledWindow, Switch, Entry, Button, Stack, ListBox, SearchEntry, MenuButton, gio};
use crate::db;
use crate::hotkey;

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
        .label("Press key combo to toggle window (e.g. <Super>v)")
        .halign(gtk::Align::Start)
        .build();
    shortcut_sub.add_css_class("settings-label-subtitle");
    shortcut_text_box.append(&shortcut_title);
    shortcut_text_box.append(&shortcut_sub);
    
    let shortcut_entry = Entry::builder()
        .valign(gtk::Align::Center)
        .width_request(150)
        .halign(gtk::Align::End)
        .build();
    
    let shortcut_spacer = Box::new(Orientation::Horizontal, 0);
    shortcut_spacer.set_hexpand(true);
    
    shortcut_card.append(&shortcut_text_box);
    shortcut_card.append(&shortcut_spacer);
    shortcut_card.append(&shortcut_entry);
    settings_box.append(&shortcut_card);

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
    let shortcut_entry_clone = shortcut_entry.clone();
    let limit_entry_clone = limit_entry.clone();

    pref_action.connect_activate(move |_, _| {
        if let Ok(conn) = db::init_db() {
            let autostart_val = db::get_config_val(&conn, "autostart")
                .unwrap_or(None)
                .unwrap_or_else(|| "true".to_string());
            let shortcut_val = db::get_config_val(&conn, "shortcut")
                .unwrap_or(None)
                .unwrap_or_else(|| "<Super>v".to_string());
            let limit_val = db::get_config_val(&conn, "history_limit")
                .unwrap_or(None)
                .unwrap_or_else(|| "200".to_string());

            autostart_switch_clone.set_active(autostart_val == "true");
            shortcut_entry_clone.set_text(&shortcut_val);
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
        let shortcut_str = shortcut_entry.text().to_string();
        let limit_str = limit_entry.text().to_string();

        if let Ok(conn) = db::init_db() {
            let _ = db::set_config_val(&conn, "autostart", autostart_str);
            let _ = db::set_config_val(&conn, "shortcut", &shortcut_str);
            let _ = db::set_config_val(&conn, "history_limit", &limit_str);

            let _ = hotkey::update_gnome_shortcut(&shortcut_str);
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
