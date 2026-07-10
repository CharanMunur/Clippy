use adw::prelude::*;
use adw::{Application, ApplicationWindow, HeaderBar, StatusPage};
use gtk::{Box, Orientation, Label, ScrolledWindow, ListBox, ListBoxRow, Stack, Picture, Button, SearchEntry};
use gtk::glib;
use chrono::{DateTime, Utc};

mod db;
mod poller;

fn format_timestamp(created_at: DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(created_at);

    if duration.num_seconds() < 60 {
        "Just now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{}m ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{}h ago", duration.num_hours())
    } else {
        created_at.format("%b %d, %H:%M").to_string()
    }
}

fn copy_to_clipboard(entry: &db::ClipboardEntry) {
    if let Some(display) = gtk::gdk::Display::default() {
        let clipboard = display.clipboard();
        if let Some(text) = &entry.text_content {
            clipboard.set_text(text);
            println!("Copied text back to clipboard (GDK): {}", text.chars().take(30).collect::<String>());
        } else if let Some(path) = &entry.image_path {
            let file = gtk::gio::File::for_path(path);
            match gtk::gdk::Texture::from_file(&file) {
                Ok(texture) => {
                    clipboard.set_texture(&texture);
                    println!("Copied image back to clipboard (GDK): {}", path);
                }
                Err(e) => {
                    eprintln!("Failed to load texture for clipboard copy: {}", e);
                }
            }
        }
    }
}

fn build_row(entry: &db::ClipboardEntry, list_box: &ListBox, stack: &Stack, search_entry: &SearchEntry) -> ListBoxRow {
    // Main Revealer wrapping the entire row's layout to animate deletion
    let main_revealer = gtk::Revealer::builder()
        .transition_type(gtk::RevealerTransitionType::SlideUp)
        .transition_duration(250)
        .reveal_child(true)
        .build();

    let row_container = Box::new(Orientation::Horizontal, 0); // No spacing here to avoid spacing gaps when actions are hidden
    row_container.set_valign(gtk::Align::Center);

    // 1. The Card Box (elevated card matching Windows 11 styling)
    let card = Box::new(Orientation::Horizontal, 0);
    card.add_css_class("card");
    card.set_hexpand(true);
    card.set_margin_bottom(4); // Spacing between rows
    if entry.text_content.is_some() {
        card.set_height_request(98); // Enforce 84px height only for text items
    }

    // Left column: content
    let left_col = Box::new(Orientation::Vertical, 4);
    left_col.set_hexpand(true);
    left_col.set_margin_top(12);
    left_col.set_margin_bottom(12);
    left_col.set_margin_start(16);
    left_col.set_margin_end(12);

    if let Some(text) = &entry.text_content {
        let preview_lines: Vec<&str> = text.lines().take(1).collect();
        let joined = preview_lines.join("\n");
        let preview_text: String = joined.chars().take(200).collect();
        
        let label = Label::builder()
            .label(&preview_text)
            .halign(gtk::Align::Fill)
            .xalign(0.0)
            .wrap(true)
            .wrap_mode(gtk::pango::WrapMode::WordChar)
            .lines(1)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .hexpand(true)
            .build();
        left_col.append(&label);
    } else if let Some(path) = &entry.image_path {
        let picture = Picture::for_filename(path);
        picture.set_height_request(120); // Dynamic height request as before
        picture.set_halign(gtk::Align::Start);
        left_col.append(&picture);
    }

    // Small timestamp at the bottom-left of the card content
    let time_label = Label::builder()
        .label(&format_timestamp(entry.created_at))
        .halign(gtk::Align::Start)
        .build();
    time_label.add_css_class("caption");
    time_label.add_css_class("dim-label");
    time_label.set_margin_top(6);
    left_col.append(&time_label);

    // Right column: More (...) and Pin buttons inside the card
    let right_col = Box::new(Orientation::Vertical, 0);
    right_col.set_valign(gtk::Align::Fill);
    right_col.set_halign(gtk::Align::End);
    right_col.set_margin_top(8);
    right_col.set_margin_bottom(8);
    right_col.set_margin_end(8);
    right_col.set_margin_start(4);

    let more_btn = Button::builder()
        .icon_name("view-more-symbolic")
        .tooltip_text("More actions")
        .build();
    more_btn.add_css_class("flat");
    more_btn.add_css_class("dim-label");

    let spacer = Box::new(Orientation::Vertical, 0);
    spacer.set_vexpand(true);

    let pin_btn = Button::builder()
        .icon_name("view-pin-symbolic")
        .tooltip_text(if entry.pinned { "Unpin entry" } else { "Pin entry" })
        .build();
    pin_btn.add_css_class("flat");
    if entry.pinned {
        pin_btn.add_css_class("suggested-action");
    } else {
        pin_btn.add_css_class("dim-label");
    }
    
    let list_box_clone = list_box.clone();
    let stack_clone = stack.clone();
    let search_entry_clone = search_entry.clone();
    let entry_id = entry.id;
    pin_btn.connect_clicked(move |_| {
        if let Ok(conn) = db::init_db() {
            let _ = db::toggle_pin_entry(&conn, entry_id);
            refresh_list(&list_box_clone, &stack_clone, &search_entry_clone);
        }
    });

    right_col.append(&more_btn);
    right_col.append(&spacer);
    right_col.append(&pin_btn);

    card.append(&left_col);
    card.append(&right_col);
    row_container.append(&card);

    // 2. Action Buttons Revealer (slides out to the right of the card, matching reference layout cp2.webp)
    let action_revealer = gtk::Revealer::builder()
        .transition_type(gtk::RevealerTransitionType::SlideLeft)
        .transition_duration(250)
        .reveal_child(false)
        .valign(gtk::Align::Fill)
        .build();

    let action_box = Box::new(Orientation::Horizontal, 0);
    action_box.set_valign(gtk::Align::Fill);
    action_box.set_margin_bottom(4); // Align with card bottom margin
    action_box.set_margin_start(0);   // Create spacing only when revealed!

    let copy_btn = Button::builder()
        .icon_name("edit-copy-symbolic")
        .tooltip_text("Copy to clipboard")
        .valign(gtk::Align::Fill)
        .halign(gtk::Align::Fill)
        .build();
    copy_btn.add_css_class("card");
    copy_btn.set_width_request(84);
    
    let entry_clone = entry.clone();
    copy_btn.connect_clicked(move |_| {
        copy_to_clipboard(&entry_clone);
    });

    let delete_btn = Button::builder()
        .icon_name("user-trash-symbolic")
        .tooltip_text("Delete entry")
        .valign(gtk::Align::Fill)
        .halign(gtk::Align::Fill)
        .build();
    delete_btn.add_css_class("card");
    delete_btn.add_css_class("error");
    delete_btn.set_width_request(84);

    let main_revealer_clone = main_revealer.clone();
    let list_box_clone = list_box.clone();
    let stack_clone = stack.clone();
    let search_entry_clone = search_entry.clone();
    delete_btn.connect_clicked(move |_| {
        // Animate row collapse
        main_revealer_clone.set_reveal_child(false);
        
        let list_box_clone = list_box_clone.clone();
        let stack_clone = stack_clone.clone();
        let search_entry_clone = search_entry_clone.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(250), move || {
            if let Ok(conn) = db::init_db() {
                let _ = db::delete_entry(&conn, entry_id);
                refresh_list(&list_box_clone, &stack_clone, &search_entry_clone);
            }
        });
    });

    action_box.append(&copy_btn);
    action_box.append(&delete_btn);
    action_revealer.set_child(Some(&action_box));
    row_container.append(&action_revealer);

    // Connect the More button to toggle the action buttons panel
    let action_revealer_clone = action_revealer.clone();
    let more_btn_clone = more_btn.clone();
    let card_clone = card.clone();
    let copy_btn_clone = copy_btn.clone();
    let delete_btn_clone = delete_btn.clone();
    more_btn.connect_clicked(move |_| {
        let is_revealed = action_revealer_clone.reveals_child();
        action_revealer_clone.set_reveal_child(!is_revealed);
        if !is_revealed {
            more_btn_clone.add_css_class("suggested-action");
            let h = card_clone.height();
            
            // Set the width request dynamically to match the height, making them perfect squares!
            copy_btn_clone.set_width_request(h);
            delete_btn_clone.set_width_request(h);
            
            // Apply capsule styling to corners
            card_clone.add_css_class("card-revealed");
            copy_btn_clone.add_css_class("btn-copy-revealed");
            delete_btn_clone.add_css_class("btn-delete-revealed");
            
            // Total panel width is h + h
            let shift = 2 * h;
            card_clone.set_margin_start(-shift);
            card_clone.set_margin_end(shift);
        } else {
            more_btn_clone.remove_css_class("suggested-action");
            card_clone.set_margin_start(0);
            card_clone.set_margin_end(0);
            
            // Revert capsule styling to corners
            card_clone.remove_css_class("card-revealed");
            copy_btn_clone.remove_css_class("btn-copy-revealed");
            delete_btn_clone.remove_css_class("btn-delete-revealed");
        }
    });

    // Drag and Drop support
    let drag_source = gtk::DragSource::new();
    drag_source.set_actions(gtk::gdk::DragAction::COPY);
    
    // Set a uniform clipboard copy icon as the drag preview for all item types
    if let Some(display) = gtk::gdk::Display::default() {
        let icon_theme = gtk::IconTheme::for_display(&display);
        let paintable = icon_theme.lookup_icon(
            "edit-copy-symbolic",
            &[],
            32, // size in px
            1,  // scale factor
            gtk::TextDirection::None,
            gtk::IconLookupFlags::empty(),
        );
        drag_source.set_icon(Some(&paintable), 16, 16);
    }
    
    if let Some(text) = &entry.text_content {
        let text_clone = text.clone();
        drag_source.connect_prepare(move |_, _, _| {
            let bytes = gtk::glib::Bytes::from(text_clone.as_bytes());
            let p1 = gtk::gdk::ContentProvider::for_bytes("text/plain", &bytes);
            let p2 = gtk::gdk::ContentProvider::for_bytes("text/plain;charset=utf-8", &bytes);
            let p3 = gtk::gdk::ContentProvider::for_bytes("UTF8_STRING", &bytes);
            let p4 = gtk::gdk::ContentProvider::for_bytes("STRING", &bytes);
            let p5 = gtk::gdk::ContentProvider::for_value(&text_clone.to_value());
            Some(gtk::gdk::ContentProvider::new_union(&[p1, p2, p3, p4, p5]))
        });
    } else if let Some(path) = &entry.image_path {
        let path_clone = path.clone();
        drag_source.connect_prepare(move |_, _, _| {
            let file = gtk::gio::File::for_path(&path_clone);
            Some(gtk::gdk::ContentProvider::for_value(&file.to_value()))
        });
    }
    card.add_controller(drag_source);

    main_revealer.set_child(Some(&row_container));
    let row = ListBoxRow::new();
    row.set_child(Some(&main_revealer));
    row.set_widget_name(&entry.id.to_string());

    row
}

fn refresh_list(list_box: &ListBox, stack: &Stack, search_entry: &SearchEntry) {
    let conn = match db::init_db() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to open DB in refresh_list: {}", e);
            return;
        }
    };

    let query = search_entry.text().to_string();
    let query_opt = if query.is_empty() { None } else { Some(query.as_str()) };

    let entries = match db::get_entries(&conn, query_opt) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to query entries in refresh_list: {}", e);
            return;
        }
    };

    // Clear list box
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }

    if entries.is_empty() {
        stack.set_visible_child_name("empty");
    } else {
        stack.set_visible_child_name("list");
        for entry in entries {
            let row = build_row(&entry, list_box, stack, search_entry);
            list_box.append(&row);
        }
    }
}

fn main() {
    let application = Application::builder()
        .application_id("org.gnome.Clippy")
        .build();

    application.connect_activate(|app| {
        // Initialize the database on startup
        if let Err(e) = db::init_db() {
            eprintln!("Failed to initialize database: {}", e);
        }

        // Initialize custom CSS provider to remove ListBox & ListBoxRow backgrounds/borders/hovers
        let provider = gtk::CssProvider::new();
        provider.load_from_data(
            "
            listbox, listboxrow {
                background-color: transparent;
                border-style: none;
                box-shadow: none;
            }
            listboxrow:hover, listboxrow:selected, listboxrow:focus, listboxrow:active {
                background-color: transparent;
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
                border-top-left-radius: 0px;
                border-bottom-left-radius: 0px;
                border-top-right-radius: 12px;
                border-bottom-right-radius: 12px;
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

        // Create a Box to hold our widgets vertically
        let content = Box::new(Orientation::Vertical, 0);

        // Header bar (empty window drag area)
        let header_bar = HeaderBar::new();

        // Pin window button on the top-left of the controls bar (always-on-top toggle)
        let pin_win_btn = Button::builder()
            .icon_name("view-pin-symbolic")
            .valign(gtk::Align::Center)
            .tooltip_text("Pin window (always on top)")
            .build();
        pin_win_btn.add_css_class("flat");
        pin_win_btn.add_css_class("dim-label");

        let pin_win_btn_clone = pin_win_btn.clone();
        let is_win_pinned = std::rc::Rc::new(std::cell::Cell::new(false));
        
        pin_win_btn.connect_clicked(move |_| {
            let new_state = !is_win_pinned.get();
            is_win_pinned.set(new_state);
            
            // Run wmctrl command to toggle always-on-top window manager state
            let action = if new_state { "add" } else { "remove" };
            let _ = std::process::Command::new("wmctrl")
                .args(&["-r", ":ACTIVE:", "-b", &format!("{},above", action)])
                .status();
            
            if new_state {
                pin_win_btn_clone.add_css_class("suggested-action");
                pin_win_btn_clone.remove_css_class("dim-label");
                pin_win_btn_clone.set_tooltip_text(Some("Unpin window (always on top)"));
            } else {
                pin_win_btn_clone.remove_css_class("suggested-action");
                pin_win_btn_clone.add_css_class("dim-label");
                pin_win_btn_clone.set_tooltip_text(Some("Pin window (always on top)"));
            }
        });
        header_bar.pack_start(&pin_win_btn);

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
        clear_all_btn.add_css_class("flat");
        clear_all_btn.add_css_class("dim-label");

        let spacer = Box::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);

        sub_header.append(&title_label);
        sub_header.append(&spacer);
        sub_header.append(&clear_all_btn);
        content.append(&sub_header);

        // Stack to toggle between Empty state and ListBox
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

        stack.add_named(&status_page, Some("empty"));
        stack.add_named(&scrolled_window, Some("list"));

        content.append(&stack);

        window.set_content(Some(&content));

        // Initial list load
        refresh_list(&list_box, &stack, &search_entry);

        // Set up "Clear all" button action
        let list_box_clone = list_box.clone();
        let stack_clone = stack.clone();
        let search_entry_clone = search_entry.clone();
        clear_all_btn.connect_clicked(move |_| {
            if let Ok(conn) = db::init_db() {
                let _ = db::clear_unpinned_entries(&conn);
                refresh_list(&list_box_clone, &stack_clone, &search_entry_clone);
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
                refresh_list(&list_box_clone, &stack_clone, &search_entry_clone);
            });
            
            *current_timer.borrow_mut() = Some(source_id);
        });

        // Set up the click handler on row activation to copy back to clipboard
        list_box.connect_row_activated(move |_, row| {
            if let Ok(id) = row.widget_name().parse::<i64>() {
                if let Ok(conn) = db::init_db() {
                    if let Ok(entries) = db::get_entries(&conn, None) {
                        if let Some(entry) = entries.iter().find(|e| e.id == id) {
                            copy_to_clipboard(entry);
                        }
                    }
                }
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
            while let Ok(_) = rx.try_recv() {
                changed = true;
            }
            if changed {
                refresh_list(&list_box_clone, &stack_clone, &search_entry_clone);
            }
            glib::ControlFlow::Continue
        });

        // Start clipboard poller
        poller::start_clipboard_poller(tx);

        window.present();
    });

    application.run();
}
