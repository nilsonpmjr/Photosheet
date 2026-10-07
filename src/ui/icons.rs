//! Módulo de Ícones Nativos do Photosheet.
//! Fornece e registra SVGs de alta resolução e simbólicos para todas as ferramentas e ações da interface.

use gtk4::gdk::Display;
use gtk4::IconTheme;
use std::fs;
use std::path::PathBuf;

const ICONS: &[(&str, &str)] = &[
    (
        "transform-move-symbolic",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" d="M8 1l3 3h-2v3h3v-2l3 3-3 3v-2h-3v3h2l-3 3-3-3h2v-3h-3v2l-3-3 3-3v2h3v-3h-2z"/></svg>"#,
    ),
    (
        "select-rectangular-symbolic",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" d="M1 1h3v1.5H1zm5 0h4v1.5H6zm6 0h3v1.5h-3zM1 6h1.5v4H1zm12.5 0H15v4h-1.5zM1 13.5h3V15H1zm5 0h4V15H6zm6 0h3V15h-3z"/></svg>"#,
    ),
    (
        "select-lasso-symbolic",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" d="M8 1C4.5 1 2 3.5 2 7c0 2.5 1.5 4.5 3.5 5.3-.2.6-.5 1.2-1 1.7-.4.4-.3 1 .1 1.3.4.3 1 .2 1.3-.2 1-1.1 1.7-2.3 2.1-3.6 3.5-.5 6-3.2 6-6.5C14 3.5 11.5 1 8 1zm0 1.5c2.7 0 4.5 1.8 4.5 4.5S10.7 11.5 8 11.5c-2.7 0-4.5-1.8-4.5-4.5S5.3 2.5 8 2.5z"/></svg>"#,
    ),
    (
        "select-magic-wand-symbolic",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" d="M12.5 1l.7 1.8L15 3.5l-1.8.7L12.5 6l-.7-1.8L10 3.5l1.8-.7zM4 3l.5 1.2L5.7 4.7l-1.2.5L4 6.4l-.5-1.2-1.2-.5 1.2-.5zm7.4 3.2l1.4 1.4-9.2 9.2-1.4-1.4z"/></svg>"#,
    ),
    (
        "tool-crop-symbolic",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" d="M4 1v3H1v1.5h3v7.5c0 .6.4 1 1 1h7.5v3H14v-3h2v-1.5H5.5V4H13c.6 0 1-.4 1-1V1h-1.5v2h-7V1z"/></svg>"#,
    ),
    (
        "draw-brush-symbolic",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" d="M13.8 1.2c-.8-.8-2.2-.8-3 0L5.3 6.7 4 8c-.4.4-.6 1-.5 1.5l.5 2c-.6.3-1.8.9-2.5 2-.4.6-.2 1.3.3 1.7.5.4 1.2.3 1.7-.1.9-.8 1.6-1.8 2-2.3l2 .5c.6.1 1.1-.1 1.5-.5l1.3-1.3 5.5-5.5c.8-.8.8-2.2 0-3z"/></svg>"#,
    ),
    (
        "draw-eraser-symbolic",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" d="M14.5 8.5L9.5 3.5 3 10l3.5 3.5h7c.6 0 1-.4 1-1v-4zm-8 4L4.4 10.4l5.1-5.1 2.1 2.1-5.1 5.1z"/></svg>"#,
    ),
    (
        "bandage-symbolic",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" d="M12.8 2.2c-1.6-1.6-4.2-1.6-5.8 0L2.2 7c-1.6 1.6-1.6 4.2 0 5.8 1.6 1.6 4.2 1.6 5.8 0l4.8-4.8c1.6-1.6 1.6-4.2 0-5.8zm-4.7 4.7l1.4 1.4-2.8 2.8-1.4-1.4 2.8-2.8z"/></svg>"#,
    ),
    (
        "clone-stamp-symbolic",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" d="M9 1H7c-.6 0-1 .4-1 1v2c0 .8.4 1.5 1 1.8V7c0 1.1-.9 2-2 2H3c-.6 0-1 .4-1 1v2h12v-2c0-.6-.4-1-1-1h-2c-1.1 0-2-.9-2-2V5.8c.6-.3 1-1 1-1.8V2c0-.6-.4-1-1-1zm5 13H2v1h12v-1z"/></svg>"#,
    ),
    (
        "blur-symbolic",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" d="M8 1S3 7.5 3 11c0 2.8 2.2 5 5 5s5-2.2 5-5c0-3.5-5-10-5-10zm0 13.5c-1.9 0-3.5-1.6-3.5-3.5 0-1.8 2.2-5.4 3.5-7.2 1.3 1.8 3.5 5.4 3.5 7.2 0 1.9-1.6 3.5-3.5 3.5z"/></svg>"#,
    ),
    (
        "color-gradient-symbolic",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" d="M2 2h12v12H2zm1.5 1.5v9h9v-9zm1.5 1.5h1.5v6H5zm3 0h1.5v6H8z"/></svg>"#,
    ),
    (
        "draw-rectangle-symbolic",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" d="M3 3h10v10H3zm1.5 1.5v7h7v-7z"/></svg>"#,
    ),
    (
        "format-text-bold-symbolic",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" d="M2 3h12v3h-4.5v7.5h-3V6H2z"/></svg>"#,
    ),
    (
        "color-picker-symbolic",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" d="M14.5 3.5l-2-2c-.7-.7-1.8-.7-2.5 0L8.7 2.8l1 1-6.9 6.9c-.3.3-.5.7-.5 1.1v2.7h2.7c.4 0 .8-.2 1.1-.5l6.9-6.9 1 1 1.5-1.5c.7-.7.7-1.8 0-2.5zm-10 10H3v-1.5l6-6 1.5 1.5-6 6z"/></svg>"#,
    ),
    (
        "open-menu-symbolic",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" d="M8 1c-.6 0-1 .4-1 1v5H6V3c0-.6-.4-1-1-1s-1 .4-1 1v6H3V5c0-.6-.4-1-1-1s-1 .4-1 1v6c0 2.8 2.2 5 5 5h3c2.8 0 5-2.2 5-5V7c0-.6-.4-1-1-1s-1 .4-1 1v2h-1V3c0-.6-.4-1-1-1s-1 .4-1 1v4H9V2c0-.6-.4-1-1-1z"/></svg>"#,
    ),
    (
        "zoom-in-symbolic",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" d="M6.5 1C3.5 1 1 3.5 1 6.5S3.5 12 6.5 12c1.3 0 2.4-.4 3.4-1.2l3.7 3.7c.4.4 1 .4 1.4 0s.4-1 0-1.4l-3.7-3.7c.8-1 1.2-2.1 1.2-3.4C12 3.5 9.5 1 6.5 1zm0 1.5c2.2 0 4 1.8 4 4s-1.8 4-4 4-4-1.8-4-4 1.8-4 4-4zm0 2v1.5H5v1h1.5V8.5h1V7H9V6H7.5V4.5z"/></svg>"#,
    ),
    (
        "edit-undo-symbolic",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" d="M6 3L1.5 7.5 6 12V9c3.3 0 6 2.7 6 6 0-5-4-9-9-9V3z"/></svg>"#,
    ),
    (
        "edit-redo-symbolic",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" d="M10 3v3C5 6 1 10 1 15c0-3.3 2.7-6 6-6v3l4.5-4.5L10 3z"/></svg>"#,
    ),
    (
        "document-send-symbolic",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" d="M8 1L4 5h3v5h2V5h3L8 1zM2 9v5h12V9h-1.5v3.5h-9V9H2z"/></svg>"#,
    ),
    (
        "object-flip-horizontal-symbolic",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" d="M4 2v2h6.5c1.4 0 2.5 1.1 2.5 2.5V8h-1.5V6.5c0-.6-.4-1-1-1H4v2L1 4.5 4 2zm8 12v-2H5.5c-1.4 0-2.5-1.1-2.5-2.5V8h1.5v1.5c0 .6.4 1 1 1H12v-2l3 2.5-3 2.5z"/></svg>"#,
    ),
    (
        "edit-clear-all-symbolic",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" d="M2 2h7v7H2V2zm5 5h7v7H7V7zm-3.5 1.5h4v4h-4z"/></svg>"#,
    ),
    (
        "image-x-generic-symbolic",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" d="M1 2v12h14V2H1zm1.5 1.5h11v9h-11v-9zm2 2a1.5 1.5 0 1 0 0 3 1.5 1.5 0 0 0 0-3zm7.5 7h-8l2.5-3 1.5 1.8 2-2.4 2 3.6z"/></svg>"#,
    ),
    (
        "folder-symbolic",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" d="M1 2v12h14V5H7L5.5 3.5 5 2H1zm1.5 3h11v7.5h-11V5z"/></svg>"#,
    ),
    (
        "folder-new-symbolic",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" d="M1 2v12h7v-1.5H2.5V5h11v3.5H15V5H7L5.5 3.5 5 2H1zm11 8v2h-2v1.5h2v2h1.5v-2h2V12h-2v-2z"/></svg>"#,
    ),
    (
        "display-brightness-symbolic",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" d="M8 1C4.1 1 1 4.1 1 8s3.1 7 7 7 7-3.1 7-7-3.1-7-7-7zm0 1.5c3 0 5.5 2.5 5.5 5.5S11 13.5 8 13.5V2.5z"/></svg>"#,
    ),
    (
        "edit-copy-symbolic",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" d="M4 1v2H2v12h9v-2h3V1H4zm1.5 1.5h7V11H12V3c0-.6-.4-1-1-1H5.5zm-2 3h6.5v8H3.5v-8z"/></svg>"#,
    ),
    (
        "layer-mask-symbolic",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" d="M1 3v10h14V3H1zm1.5 1.5h11v7h-11v-7zm5.5 1a2.5 2.5 0 1 0 0 5 2.5 2.5 0 0 0 0-5z"/></svg>"#,
    ),
    (
        "edit-clear-symbolic",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" d="M3 3l10 10M13 3L3 13"/></svg>"#,
    ),
];

pub fn init_icon_theme() {
    let display = match Display::default() {
        Some(d) => d,
        None => return,
    };

    let theme = IconTheme::for_display(&display);

    // 1. Extrai os ícones para o diretório de cache em runtime
    let cache_dir = get_icons_cache_dir();
    let _ = fs::create_dir_all(&cache_dir);

    for (name, svg) in ICONS {
        let file_path = cache_dir.join(format!("{}.svg", name));
        if !file_path.exists() {
            let _ = fs::write(&file_path, svg);
        }
    }

    // 2. Registra o diretório de cache no GTK IconTheme
    theme.add_search_path(&cache_dir);

    // 3. Registra diretórios do projeto e do sistema para fallback abrangente
    theme.add_search_path("data/icons/tools");
    theme.add_search_path("data/icons/hicolor/scalable/actions");

    if let Ok(home) = std::env::var("HOME") {
        theme.add_search_path(PathBuf::from(&home).join(".local/share/icons/hicolor/scalable/actions"));
        theme.add_search_path(PathBuf::from(&home).join(".local/share/icons"));
    }

    // Diretórios de temas padrão no Linux
    let system_paths = [
        "/usr/share/icons/Adwaita/symbolic/actions",
        "/usr/share/icons/Adwaita/symbolic/ui",
        "/usr/share/icons/Adwaita/symbolic/status",
        "/usr/share/icons/Adwaita/symbolic/legacy",
        "/usr/share/icons/hicolor",
        "/usr/share/icons/Papirus/24x24/actions",
        "/usr/share/icons/breeze/actions/24",
    ];
    for p in system_paths {
        let pb = PathBuf::from(p);
        if pb.exists() {
            theme.add_search_path(pb);
        }
    }
}

fn get_icons_cache_dir() -> PathBuf {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(runtime_dir).join("photosheet-icons")
    } else {
        std::env::temp_dir().join("photosheet-icons")
    }
}
