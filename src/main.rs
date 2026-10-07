pub mod core;
pub mod ffi;
pub mod io;
pub mod render;
pub mod ui;

use gtk4::prelude::*;
use ui::dialogs::NewDocumentParams;
use ui::PhotosheetWindow;

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
    let window = PhotosheetWindow::new(app);
    // Cria um documento padrão inicial Full HD
    window.create_new_document(NewDocumentParams {
        width: 1920,
        height: 1080,
        resolution: 72.0,
        background_color: Some([255, 255, 255, 255]),
    });
    window.present();
}

