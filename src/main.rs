pub mod core;
pub mod ffi;
pub mod io;
pub mod render;

use gtk4::prelude::*;
use libadwaita::prelude::*;

const APP_ID: &str = "org.photosheet.Photosheet";

fn main() {
    let app = libadwaita::Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);

    // Runs application loop
    app.run();
}

fn build_ui(app: &libadwaita::Application) {
    let window = libadwaita::ApplicationWindow::builder()
        .application(app)
        .title("Photosheet")
        .default_width(1280)
        .default_height(800)
        .build();

    let header_bar = libadwaita::HeaderBar::new();
    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    content.append(&header_bar);

    let label = gtk4::Label::builder()
        .label("Photosheet — Linux Native Image Editor")
        .vexpand(true)
        .hexpand(true)
        .build();
    content.append(&label);

    window.set_content(Some(&content));
    window.present();
}
