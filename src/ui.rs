use adw::prelude::*;
use gtk::{Box, Orientation, Label, ListBox, ListBoxRow, Stack, Picture, Button, SearchEntry};
use gtk::glib;
use chrono::{DateTime, Utc};
use crate::db::{self, ClipboardEntry};

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

pub fn copy_to_clipboard(entry: &ClipboardEntry) {
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
                    eprintln!("Failed to load texture from file {}: {}", path, e);
                }
            }
        }
    }
}

pub fn clear_clipboard() {
    if let Some(display) = gtk::gdk::Display::default() {
        let clipboard = display.clipboard();
        clipboard.set_text("");
        println!("Cleared system clipboard");
    }
}

pub fn build_row(
    entry: &ClipboardEntry,
    list_box: &ListBox,
    stack: &Stack,
    search_entry: &SearchEntry,
) -> ListBoxRow {
    let list_row = ListBoxRow::new();
    list_row.set_widget_name(&entry.id.to_string());

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
    card.set_tooltip_text(Some("Click to copy"));
    card.set_cursor_from_name(Some("pointer"));
    if entry.text_content.is_some() {
        card.set_height_request(98); // Enforce fixed height for text items
    } else {
        card.set_height_request(110); // Cap image cards to prevent window expansion
    }

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
        let (img_w, img_h) = if let Ok((w, h)) = image::image_dimensions(path) {
            (w as f64, h as f64)
        } else {
            (100.0, 100.0) // Fallback aspect ratio 1.0
        };
        let aspect = img_w / img_h;
        let target_h = 80.0;
        // Bound target width to max 200px to prevent card horizontal overflow
        let target_w = (target_h * aspect).min(200.0);

        let picture = Picture::for_filename(path);
        picture.set_height_request(target_h as i32);
        picture.set_width_request(target_w as i32);
        picture.set_can_shrink(true);
        picture.set_halign(gtk::Align::Start);
        picture.set_valign(gtk::Align::Center);

        // Wrap in a horizontal box to force left-alignment in vertical left_col
        let img_wrapper = Box::new(Orientation::Horizontal, 0);
        img_wrapper.set_halign(gtk::Align::Start);
        img_wrapper.append(&picture);
        left_col.append(&img_wrapper);
    }

    // Bottom spacer inside left column to push timestamp to the bottom, aligning with right-col pin button
    let spacer = Box::new(Orientation::Vertical, 0);
    spacer.set_vexpand(true);
    left_col.append(&spacer);

    // Timestamp under content
    let time_label = Label::builder()
        .label(format_timestamp(entry.created_at))
        .halign(gtk::Align::Start)
        .build();
    time_label.add_css_class("dim-label");
    time_label.add_css_class("caption");
    left_col.append(&time_label);

    card.append(&left_col);

    // Right column: action buttons (More actions button, Pinned button)
    let right_col = Box::new(Orientation::Vertical, 4);
    right_col.set_margin_top(12);
    right_col.set_margin_bottom(12);
    right_col.set_margin_start(12);
    right_col.set_margin_end(16);
    right_col.set_valign(gtk::Align::Fill);
    right_col.set_halign(gtk::Align::End);

    // Toggle button for actions panel (sliding out)
    let more_btn = Button::builder()
        .icon_name("view-more-symbolic")
        .tooltip_text("More actions")
        .build();
    more_btn.add_css_class("flat");
    more_btn.add_css_class("dim-label");

    let spacer = Box::new(Orientation::Vertical, 0);
    spacer.set_vexpand(true);

    let pin_icon = if entry.pinned {
        "clippy-pin-active-symbolic"
    } else {
        "clippy-pin-symbolic"
    };

    let pin_btn = Button::builder()
        .icon_name(pin_icon)
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

    card.append(&right_col);
    row_container.append(&card);

    // 2. Sliding Actions Panel (wrapped in a Revealer to slide in/out horizontally)
    let action_revealer = gtk::Revealer::builder()
        .transition_type(gtk::RevealerTransitionType::SlideLeft)
        .transition_duration(150)
        .reveal_child(false)
        .build();

    let action_box = Box::new(Orientation::Horizontal, 0);
    action_box.set_margin_start(4); // 4px gap between card and action buttons

    // Define delete_btn first
    let delete_btn = Button::builder()
        .icon_name("user-trash-symbolic")
        .tooltip_text("Delete entry")
        .valign(gtk::Align::Fill)
        .halign(gtk::Align::Fill)
        .build();
    delete_btn.add_css_class("card");
    delete_btn.set_width_request(84);

    // Left click on item copies directly and refreshes list instantly
    let gesture = gtk::GestureClick::new();
    let entry_clone = entry.clone();
    let list_box_clone_copy = list_box.clone();
    let stack_clone_copy = stack.clone();
    let search_entry_clone_copy = search_entry.clone();
    gesture.connect_released(move |gesture, _, _, _| {
        if gesture.current_button() == gtk::gdk::BUTTON_PRIMARY {
            copy_to_clipboard(&entry_clone);
            
            // Instantly update database and refresh UI
            if let Ok(conn) = db::init_db() {
                let hash = entry_clone.content_hash.clone();
                let _ = db::insert_entry(
                    &conn,
                    entry_clone.kind,
                    entry_clone.text_content.as_deref(),
                    entry_clone.image_path.as_deref(),
                    &hash,
                );
                refresh_list(&list_box_clone_copy, &stack_clone_copy, &search_entry_clone_copy);
            }
        }
    });
    card.add_controller(gesture);

    action_box.append(&delete_btn);
    action_revealer.set_child(Some(&action_box));
    row_container.append(&action_revealer);

    // Connect the More button to toggle the action buttons panel
    let action_revealer_clone = action_revealer.clone();
    let more_btn_clone = more_btn.clone();
    let card_clone = card.clone();
    let delete_btn_clone = delete_btn.clone();
    let list_box_clone = list_box.clone();
    let is_text = entry.text_content.is_some();
    more_btn.connect_clicked(move |_| {
        let is_revealed = action_revealer_clone.reveals_child();
        
        if !is_revealed {
            // Close all other revealed action panels in the list first
            let mut sibling = list_box_clone.first_child();
            while let Some(row_widget) = sibling {
                if let Some(row) = row_widget.downcast_ref::<ListBoxRow>()
                    && let Some(main_rev) = row.child().and_then(|w| w.downcast::<gtk::Revealer>().ok())
                        && let Some(container) = main_rev.child().and_then(|w| w.downcast::<Box>().ok()) {
                            let mut child = container.first_child();
                            let mut found_card: Option<Box> = None;
                            let mut found_revealer: Option<gtk::Revealer> = None;
                            while let Some(w) = child {
                                if let Some(c) = w.downcast_ref::<Box>() {
                                    if c.has_css_class("card") {
                                        found_card = Some(c.clone());
                                    }
                                } else if let Some(r) = w.downcast_ref::<gtk::Revealer>() {
                                    found_revealer = Some(r.clone());
                                }
                                child = w.next_sibling();
                            }
                            
                            if let (Some(c), Some(r)) = (found_card, found_revealer)
                                && r != action_revealer_clone {
                                    r.set_reveal_child(false);
                                    c.remove_css_class("card-revealed");
                                    
                                    if let Some(action_box) = r.child().and_then(|w| w.downcast::<Box>().ok()) {
                                        let mut btn_child = action_box.first_child();
                                        while let Some(btn) = btn_child {
                                            if let Some(b) = btn.downcast_ref::<Button>() {
                                                b.remove_css_class("btn-copy-revealed");
                                                b.remove_css_class("btn-delete-revealed");
                                            }
                                            btn_child = btn.next_sibling();
                                        }
                                    }
                                    
                                    let mut card_child = c.first_child();
                                    while let Some(cc) = card_child {
                                        if let Some(right_c) = cc.downcast_ref::<Box>()
                                            && let Some(m_btn) = right_c.first_child().and_then(|w| w.downcast::<Button>().ok()) {
                                                m_btn.remove_css_class("suggested-action");
                                            }
                                        card_child = cc.next_sibling();
                                    }
                                }
                        }
                sibling = row_widget.next_sibling();
            }
        }

        action_revealer_clone.set_reveal_child(!is_revealed);
        if !is_revealed {
            more_btn_clone.add_css_class("suggested-action");
            let target_w = if is_text { 98 } else { 110 };
            delete_btn_clone.set_width_request(target_w);
            
            // Apply capsule corner styling
            card_clone.add_css_class("card-revealed");
            delete_btn_clone.add_css_class("btn-delete-revealed");
        } else {
            more_btn_clone.remove_css_class("suggested-action");
            
            // Revert capsule corner styling
            card_clone.remove_css_class("card-revealed");
            delete_btn_clone.remove_css_class("btn-delete-revealed");
        }
    });

    // Connect Delete button click with fade/slide out animation, then database deletion
    let main_revealer_clone = main_revealer.clone();
    let entry_id = entry.id;
    let list_box_clone_del = list_box.clone();
    let stack_clone_del = stack.clone();
    let search_entry_clone_del = search_entry.clone();
    delete_btn.connect_clicked(move |_| {
        main_revealer_clone.set_reveal_child(false);
        
        let list_box_clone = list_box_clone_del.clone();
        let stack_clone = stack_clone_del.clone();
        let search_entry_clone = search_entry_clone_del.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(250), move || {
            if let Ok(conn) = db::init_db() {
                let _ = db::delete_entry(&conn, entry_id);
                
                // Keep system clipboard in sync with the new top entry
                let entries = db::get_entries(&conn, None).unwrap_or_default();
                if let Some(first_entry) = entries.first() {
                    copy_to_clipboard(first_entry);
                } else {
                    clear_clipboard();
                }

                refresh_list(&list_box_clone, &stack_clone, &search_entry_clone);
            }
        });
    });

    main_revealer.set_child(Some(&row_container));
    list_row.set_child(Some(&main_revealer));

    list_row
}

pub fn refresh_list(list_box: &ListBox, stack: &Stack, search_entry: &SearchEntry) {
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
