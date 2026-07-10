use adw::prelude::*;
use adw::{Application, ApplicationWindow, HeaderBar};
use gtk::{Box, Orientation, Label};

fn main() {
    let application = Application::builder()
        .application_id("org.gnome.Clippy")
        .build();

    application.connect_activate(|app| {
        // Create a Window
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Clippy")
            .default_width(480)
            .default_height(600)
            .build();

        // Create a Box to hold our widgets vertically
        let content = Box::new(Orientation::Vertical, 0);

        // Header bar
        let header_bar = HeaderBar::new();
        content.append(&header_bar);

        // A placeholder label
        let label = Label::builder()
            .label("Clippy Clipboard Manager")
            .valign(gtk::Align::Center)
            .halign(gtk::Align::Center)
            .vexpand(true)
            .hexpand(true)
            .build();
        
        // Add CSS class for styling
        label.add_css_class("title-1");

        content.append(&label);

        window.set_content(Some(&content));
        window.present();
    });

    application.run();
}
