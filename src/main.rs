use adw::prelude::*;
use adw::{Application, ApplicationWindow, HeaderBar, StatusPage};
use gtk::{Box, Orientation, Label, ScrolledWindow, ListBox, Stack, Button, SearchEntry, MenuButton, AboutDialog, MessageDialog, gio};
use gtk::glib;

thread_local! {
    static WINDOW: std::cell::RefCell<Option<ApplicationWindow>> = const { std::cell::RefCell::new(None) };
    static HOLD_GUARD: std::cell::RefCell<Option<gtk::gio::ApplicationHoldGuard>> = const { std::cell::RefCell::new(None) };
    static DIALOG_OPEN: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    // Tracks whether the window is pinned always-on-top so we can re-apply after re-present
    static WIN_PINNED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

mod db;
mod poller;
mod hotkey;
mod settings;
mod ui;

fn build_ui(app: &Application) -> ApplicationWindow {
    // Initialize the database on startup
    if let Err(e) = db::init_db() {
        eprintln!("Failed to initialize database: {}", e);
    }

    // Add custom icons search path — installed location + dev fallback
    if let Some(display) = gtk::gdk::Display::default() {
        let icon_theme = gtk::IconTheme::for_display(&display);
        // Installed: ~/.local/share/clippy/icons
        if let Some(proj) = directories::ProjectDirs::from("com", "clippy", "clippy") {
            let mut data = proj.data_dir().to_path_buf();
            data.push("icons");
            icon_theme.add_search_path(&data);
        }
        // Dev fallback: <project_root>/icons
        if let Ok(mut cwd) = std::env::current_dir() {
            cwd.push("icons");
            icon_theme.add_search_path(&cwd);
        }
    }

    // Initialize custom CSS provider to remove ListBox & ListBoxRow backgrounds/borders/hovers
    let provider = gtk::CssProvider::new();
    provider.load_from_data(
        "
        listbox, listboxrow {
            background-color: transparent;
            border-style: none;
            box-shadow: none;
            padding: 0;
            margin: 0;
        }
        listboxrow:hover, listboxrow:selected, listboxrow:focus, listboxrow:active {
            background-color: transparent;
        }
        .card {
            border-radius: 6px;
            background-color: @card_bg_color;
            border: 1.5px solid alpha(@window_fg_color, 0.10);
            box-shadow: 0 2px 4px alpha(black, 0.04);
        }
        .card.card-revealed {
            border-top-right-radius: 0px;
            border-bottom-right-radius: 0px;
        }
        button.card.btn-copy-revealed {
            border-top-left-radius: 0px;
            border-bottom-left-radius: 0px;
            border-top-right-radius: 0px;
            border-bottom-right-radius: 0px;
        }
        button.card.btn-delete-revealed {
            background-color: transparent;
            border: 1px solid alpha(@window_fg_color, 0.05);
            color: alpha(@window_fg_color, 0.6);
            border-top-left-radius: 0px;
            border-bottom-left-radius: 0px;
            border-top-right-radius: 6px;
            border-bottom-right-radius: 6px;
            box-shadow: none;
            transition: border-color 0.2s, color 0.2s;
        }
        button.card.btn-delete-revealed:hover {
            background-color: alpha(@window_fg_color, 0.08);
            border-color: alpha(@accent_color, 0.4);
            color: @accent_color;
        }
        button.card.btn-delete-revealed:active {
            background-color: alpha(@window_fg_color, 0.04);
        }
        .clear-all-btn {
            background-color: @accent_bg_color;
            color: @accent_fg_color;
            border-radius: 6px;
            padding: 4px 10px;
            font-weight: 500;
            border: none;
        }
        .clear-all-btn:hover {
            background-color: @accent_bg_color;
            color: @accent_fg_color;
            opacity: 0.9;
        }
        .clear-all-btn:active {
            opacity: 0.8;
        }
        button.suggested-action, button.flat.suggested-action {
            background-color: transparent;
            color: @accent_color;
            border: none;
            box-shadow: none;
        }
        button.suggested-action:hover, button.flat.suggested-action:hover {
            background-color: alpha(@window_fg_color, 0.08);
            color: @accent_color;
        }
        .settings-card {
            background-color: alpha(@window_fg_color, 0.03);
            border: 1px solid alpha(@window_fg_color, 0.08);
            border-radius: 8px;
            padding: 14px 18px;
            margin-bottom: 12px;
        }
        .settings-label-title {
            font-weight: 600;
            font-size: 14px;
            color: @window_fg_color;
        }
        .settings-label-subtitle {
            font-size: 12px;
            color: alpha(@window_fg_color, 0.55);
        }
        "
    );
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    // Create a Window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Clippy")
        .default_width(480)
        .default_height(600)
        .resizable(false)
        .build();

    // Intercept close request: set up after pin_win_btn (done below)
    // See connect_close_request further down

    // Create a Box to hold our widgets vertically
    let content = Box::new(Orientation::Vertical, 0);

    // Header bar (empty window drag area)
    let header_bar = HeaderBar::new();

    // Custom Title Label in the center of the header bar
    let header_title = Label::builder()
        .label("Clippy")
        .build();
    header_title.add_css_class("title-4");
    header_bar.set_title_widget(Some(&header_title));

    // Back button on the top-left of the controls bar (back to list, hidden by default)
    let back_btn = Button::builder()
        .icon_name("go-previous-symbolic")
        .valign(gtk::Align::Center)
        .tooltip_text("Back to clipboard")
        .visible(false)
        .build();
    back_btn.add_css_class("flat");
    back_btn.add_css_class("dim-label");
    header_bar.pack_start(&back_btn);

    // Pin window button on the top-left of the controls bar (always-on-top toggle)
    let pin_win_btn = Button::builder()
        .icon_name("clippy-pin-symbolic")
        .valign(gtk::Align::Center)
        .tooltip_text("Pin window (always on top)")
        .build();
    pin_win_btn.add_css_class("flat");
    pin_win_btn.add_css_class("dim-label");

    let pin_win_btn_clone = pin_win_btn.clone();

    pin_win_btn.connect_clicked(move |_| {
        let new_state = WIN_PINNED.with(|p| {
            let v = !p.get();
            p.set(v);
            v
        });

        // Apply always-on-top via wmctrl
        let action = if new_state { "add" } else { "remove" };
        let _ = std::process::Command::new("wmctrl")
            .args(["-r", ":ACTIVE:", "-b", &format!("{},above", action)])
            .status();

        if new_state {
            pin_win_btn_clone.add_css_class("suggested-action");
            pin_win_btn_clone.remove_css_class("dim-label");
            pin_win_btn_clone.set_icon_name("clippy-pin-active-symbolic");
            pin_win_btn_clone.set_tooltip_text(Some("Unpin window (always on top)"));
        } else {
            pin_win_btn_clone.remove_css_class("suggested-action");
            pin_win_btn_clone.add_css_class("dim-label");
            pin_win_btn_clone.set_icon_name("clippy-pin-symbolic");
            pin_win_btn_clone.set_tooltip_text(Some("Pin window (always on top)"));
        }
    });
    header_bar.pack_start(&pin_win_btn);

    // Intercept close request: hide window and reset pin state
    let pin_reset_btn = pin_win_btn.clone();
    window.connect_close_request(move |win| {
        if WIN_PINNED.with(|p| p.get()) {
            WIN_PINNED.with(|p| p.set(false));
            let _ = std::process::Command::new("wmctrl")
                .args(["-r", ":ACTIVE:", "-b", "remove,above"])
                .status();
            pin_reset_btn.remove_css_class("suggested-action");
            pin_reset_btn.add_css_class("dim-label");
            pin_reset_btn.set_icon_name("clippy-pin-symbolic");
            pin_reset_btn.set_tooltip_text(Some("Pin window (always on top)"));
        }
        win.hide();
        gtk::glib::Propagation::Stop
    });

    // Create GMenu for the MenuButton
    let menu = gio::Menu::new();
    menu.append(Some("Preferences"), Some("app.preferences"));
    menu.append(Some("Keyboard Shortcuts"), Some("app.shortcuts"));
    menu.append(Some("About Clippy"), Some("app.about"));

    // Menu button on the top-right of the controls bar (hamburger menu)
    let menu_button = MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .menu_model(&menu)
        .valign(gtk::Align::Center)
        .tooltip_text("Menu")
        .build();
    menu_button.add_css_class("flat");
    menu_button.add_css_class("dim-label");
    header_bar.pack_end(&menu_button);

    content.append(&header_bar);

    // Live Search Entry Bar (replacing the tab bar)
    let search_entry = SearchEntry::builder()
        .margin_start(16)
        .margin_end(16)
        .margin_top(8)
        .margin_bottom(8)
        .placeholder_text("Search clipboard...")
        .build();
    content.append(&search_entry);

    // Sub-Header Box (Clipboard label on left, Clear all on right)
    let sub_header = Box::new(Orientation::Horizontal, 0);
    sub_header.set_margin_top(8);
    sub_header.set_margin_bottom(8);
    sub_header.set_margin_start(18);
    sub_header.set_margin_end(16);

    let title_label = Label::builder()
        .label("Clipboard")
        .halign(gtk::Align::Start)
        .build();
    title_label.add_css_class("title-4");

    let clear_all_btn = Button::builder()
        .label("Clear all")
        .valign(gtk::Align::Center)
        .tooltip_text("Clear all unpinned entries")
        .build();
    clear_all_btn.add_css_class("clear-all-btn");

    let spacer = Box::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);

    sub_header.append(&title_label);
    sub_header.append(&spacer);
    sub_header.append(&clear_all_btn);
    content.append(&sub_header);

    // Stack to toggle between Empty state, ListBox, and Settings view
    let stack = Stack::builder()
        .transition_type(gtk::StackTransitionType::Crossfade)
        .vexpand(true)
        .build();

    // Empty state StatusPage
    let status_page = StatusPage::builder()
        .title("No Copied Items")
        .description("Copied text or images will appear here")
        .icon_name("edit-copy-symbolic")
        .vexpand(true)
        .build();

    // Scrolled list box
    let scrolled_window = ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .vexpand(true)
        .build();

    let list_box = ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    scrolled_window.set_child(Some(&list_box));

    // Build Settings Page and connect handlers
    let settings_scrolled = settings::build_settings_view(
        app,
        &stack,
        &list_box,
        &search_entry,
        &sub_header,
        &menu_button,
        &pin_win_btn,
        &back_btn,
        &header_title,
        ui::refresh_list,
    );

    stack.add_named(&status_page, Some("empty"));
    stack.add_named(&scrolled_window, Some("list"));
    stack.add_named(&settings_scrolled, Some("settings"));

    content.append(&stack);

    window.set_content(Some(&content));

    // Initial list load
    ui::refresh_list(&list_box, &stack, &search_entry);

    // Set up "Clear all" button action
    let list_box_clone = list_box.clone();
    let stack_clone = stack.clone();
    let search_entry_clone = search_entry.clone();
    clear_all_btn.connect_clicked(move |_| {
        if let Ok(conn) = db::init_db() {
            let _ = db::clear_unpinned_entries(&conn);
            ui::refresh_list(&list_box_clone, &stack_clone, &search_entry_clone);
        }
    });

    // Track the current debounce timer ID
    let current_timer = std::rc::Rc::new(std::cell::RefCell::new(None::<glib::SourceId>));
    
    let list_box_clone2 = list_box.clone();
    let stack_clone2 = stack.clone();
    let search_entry_clone2 = search_entry.clone();
    
    search_entry.connect_search_changed(move |_| {
        // Cancel the previous timer if it exists
        if let Some(source_id) = current_timer.borrow_mut().take() {
            source_id.remove();
        }
        
        let list_box_clone = list_box_clone2.clone();
        let stack_clone = stack_clone2.clone();
        let search_entry_clone = search_entry_clone2.clone();
        let current_timer_clone = current_timer.clone();
        
        // Start a new timer for 250ms
        let source_id = glib::timeout_add_local_once(std::time::Duration::from_millis(250), move || {
            current_timer_clone.borrow_mut().take();
            ui::refresh_list(&list_box_clone, &stack_clone, &search_entry_clone);
        });
        
        *current_timer.borrow_mut() = Some(source_id);
    });

    // Set up the click handler on row activation to copy back to clipboard
    let list_box_clone_act = list_box.clone();
    let stack_clone_act = stack.clone();
    let search_entry_clone_act = search_entry.clone();
    list_box.connect_row_activated(move |_, row| {
        if let Ok(id) = row.widget_name().parse::<i64>()
            && let Ok(conn) = db::init_db()
                && let Ok(entries) = db::get_entries(&conn, None)
                    && let Some(entry) = entries.iter().find(|e| e.id == id) {
                        ui::copy_to_clipboard(entry);
                        
                        // Instantly update database and refresh UI
                        let hash = entry.content_hash.clone();
                        let _ = db::insert_entry(
                            &conn,
                            entry.kind,
                            entry.text_content.as_deref(),
                            entry.image_path.as_deref(),
                            &hash,
                        );
                        ui::refresh_list(&list_box_clone_act, &stack_clone_act, &search_entry_clone_act);
                    }
    });

    // Set up communication channel between background poller and main thread
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    
    // Poll the channel in the GLib event loop every 100ms
    let list_box_clone = list_box.clone();
    let stack_clone = stack.clone();
    let search_entry_clone = search_entry.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        let mut changed = false;
        while rx.try_recv().is_ok() {
            changed = true;
        }
        if changed {
            ui::refresh_list(&list_box_clone, &stack_clone, &search_entry_clone);
        }
        glib::ControlFlow::Continue
    });

    // Start clipboard poller
    poller::start_clipboard_poller(tx);

    // Register "Keyboard Shortcuts" action — guard against duplicate dialogs
    let shortcuts_action = gio::SimpleAction::new("shortcuts", None);
    let win_clone = window.clone();
    shortcuts_action.connect_activate(move |_, _| {
        let already_open = DIALOG_OPEN.with(|f| f.get());
        if already_open { return; }
        DIALOG_OPEN.with(|f| f.set(true));

        let dialog = MessageDialog::builder()
            .transient_for(&win_clone)
            .modal(true)
            .message_type(gtk::MessageType::Info)
            .buttons(gtk::ButtonsType::Close)
            .text("Keyboard Shortcuts")
            .secondary_text("Global Shortcuts:\n  • Toggle Window:  Super + J  (default)\n      Change it: ☰ Menu → Preferences → Global Shortcut → click to record\n\nIn-App Actions:\n  • Copy item         Left-click any card\n  • Pin / Unpin       Click the pin icon on a card\n  • Open actions      Click ··· on a card\n  • Delete item       Open ··· → click the bin\n  • Always on top     Click the pin icon in the header\n  • Search            Type in the search bar (250 ms debounce)")
            .build();

        dialog.connect_response(|dialog, _| {
            DIALOG_OPEN.with(|f| f.set(false));
            dialog.destroy();
        });
        dialog.present();
    });
    app.add_action(&shortcuts_action);

    // Register "About Clippy" action — guard against duplicate dialogs
    let about_action = gio::SimpleAction::new("about", None);
    let win_clone = window.clone();
    about_action.connect_activate(move |_, _| {
        let already_open = DIALOG_OPEN.with(|f| f.get());
        if already_open { return; }
        DIALOG_OPEN.with(|f| f.set(true));

        let about = AboutDialog::builder()
            .transient_for(&win_clone)
            .modal(true)
            .program_name("Clippy")
            .logo_icon_name("io.github.CharanMunur.Clippy")
            .version("0.1.0")
            .comments("A native clipboard manager for GNOME/Linux")
            .website("https://github.com/CharanMunur/Clippy")
            .website_label("GitHub Repository")
            .build();
        about.set_authors(&["Charan Munur"]);
        about.add_credit_section("Developer Portfolio", &["https://www.charanmunur.in"]);

        about.connect_destroy(|_| {
            DIALOG_OPEN.with(|f| f.set(false));
        });
        about.present();
    });
    app.add_action(&about_action);

    // Initialize GNOME global shortcut and autostart configuration on startup based on DB
    if let Ok(conn) = db::init_db() {
        let shortcut = db::get_config_val(&conn, "shortcut")
            .unwrap_or(None)
            .unwrap_or_else(|| "<Super>j".to_string());
        let autostart_val = db::get_config_val(&conn, "autostart")
            .unwrap_or(None)
            .unwrap_or_else(|| "true".to_string());
        
        let _ = hotkey::update_gnome_shortcut(&shortcut);
        let _ = settings::set_autostart_enabled(autostart_val == "true");
    }

    window
}

fn main() {
    let application = Application::builder()
        .application_id("io.github.CharanMunur.Clippy")
        .flags(gtk::gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();

    // Register options so GTK doesn't exit on '--background' or '--toggle'
    application.add_main_option(
        "background",
        glib::Char::from(b'b'),
        glib::OptionFlags::NONE,
        glib::OptionArg::None,
        "Start application in the background",
        None,
    );
    application.add_main_option(
        "toggle",
        glib::Char::from(b't'),
        glib::OptionFlags::NONE,
        glib::OptionArg::None,
        "Toggle window visibility",
        None,
    );

    application.connect_command_line(move |app, app_cmd| {
        let args = app_cmd.arguments();
        let is_background = args.iter().any(|arg| arg == std::ffi::OsStr::new("--background") || arg == std::ffi::OsStr::new("-b"));
        let is_toggle = args.iter().any(|arg| arg == std::ffi::OsStr::new("--toggle") || arg == std::ffi::OsStr::new("-t"));

        WINDOW.with(|win_cell| {
            let mut win_opt = win_cell.borrow_mut();
            if let Some(win) = win_opt.as_ref() {
                // Secondary instance launched, handle toggle/present signals
                if is_toggle {
                    if win.is_visible() {
                        // Reset pin before hiding
                        if WIN_PINNED.with(|p| p.get()) {
                            win.close(); // triggers connect_close_request which resets pin
                        } else {
                            win.hide();
                        }
                    } else {
                        win.present();
                    }
                } else if !is_background {
                    win.present();
                }
            } else {
                // Primary instance startup, build the UI in background or active
                let win = build_ui(app);
                *win_opt = Some(win.clone());
                let guard = app.hold();
                HOLD_GUARD.with(|g| *g.borrow_mut() = Some(guard));
                if !is_background {
                    win.present();
                }
            }
        });
        gtk::glib::ExitCode::from(0)
    });

    application.run();
}
