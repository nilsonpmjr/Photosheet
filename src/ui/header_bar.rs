//! Barra de Cabeçalho Libadwaita (HeaderBar).
//! Traduzido de Compositor/UI/NavigationToolHeader.swift.

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

pub struct PhotosheetHeaderBar {
    pub header_bar: libadwaita::HeaderBar,
    pub title_label: Label,
    pub zoom_label: Label,
    pub undo_btn: Button,
    pub redo_btn: Button,
    on_new_clicked: Rc<RefCell<Option<Box<dyn Fn()>>>>,
    on_open_clicked: Rc<RefCell<Option<Box<dyn Fn()>>>>,
    on_save_clicked: Rc<RefCell<Option<Box<dyn Fn()>>>>,
    on_export_clicked: Rc<RefCell<Option<Box<dyn Fn()>>>>,
    on_undo_clicked: Rc<RefCell<Option<Box<dyn Fn()>>>>,
    on_redo_clicked: Rc<RefCell<Option<Box<dyn Fn()>>>>,
}

impl PhotosheetHeaderBar {
    pub fn new() -> Self {
        let header_bar = libadwaita::HeaderBar::new();

        // Botões à Esquerda: Novo, Abrir, Salvar
        let left_box = GtkBox::new(Orientation::Horizontal, 4);

        let new_btn = Button::builder()
            .icon_name("document-new-symbolic")
            .tooltip_text("Novo Documento (Ctrl+N)")
            .build();
        new_btn.add_css_class("flat");

        let open_btn = Button::builder()
            .icon_name("document-open-symbolic")
            .tooltip_text("Abrir Imagem / Projeto (Ctrl+O)")
            .build();
        open_btn.add_css_class("flat");

        let save_btn = Button::builder()
            .icon_name("document-save-symbolic")
            .tooltip_text("Salvar Projeto (Ctrl+S)")
            .build();
        save_btn.add_css_class("flat");

        left_box.append(&new_btn);
        left_box.append(&open_btn);
        left_box.append(&save_btn);

        header_bar.pack_start(&left_box);

        // Centro: Título do Documento e Indicador de Zoom
        let title_box = GtkBox::new(Orientation::Horizontal, 8);
        title_box.set_halign(gtk4::Align::Center);

        let title_label = Label::builder()
            .label("Sem Título")
            .css_classes(["title"])
            .build();

        let zoom_label = Label::builder()
            .label("100%")
            .css_classes(["dim-label", "caption"])
            .build();

        title_box.append(&title_label);
        title_box.append(&zoom_label);

        header_bar.set_title_widget(Some(&title_box));

        // Botões à Direita: Desfazer, Refazer, Exportar
        let right_box = GtkBox::new(Orientation::Horizontal, 4);

        let undo_btn = Button::builder()
            .icon_name("edit-undo-symbolic")
            .tooltip_text("Desfazer (Ctrl+Z)")
            .build();
        undo_btn.add_css_class("flat");

        let redo_btn = Button::builder()
            .icon_name("edit-redo-symbolic")
            .tooltip_text("Refazer (Ctrl+Shift+Z)")
            .build();
        redo_btn.add_css_class("flat");

        let export_btn = Button::builder()
            .icon_name("document-send-symbolic")
            .tooltip_text("Exportar Imagem (Ctrl+E)")
            .build();
        export_btn.add_css_class("flat");

        right_box.append(&undo_btn);
        right_box.append(&redo_btn);
        right_box.append(&export_btn);

        header_bar.pack_end(&right_box);

        let on_new_clicked = Rc::new(RefCell::new(None::<Box<dyn Fn()>>));
        let on_open_clicked = Rc::new(RefCell::new(None::<Box<dyn Fn()>>));
        let on_save_clicked = Rc::new(RefCell::new(None::<Box<dyn Fn()>>));
        let on_export_clicked = Rc::new(RefCell::new(None::<Box<dyn Fn()>>));
        let on_undo_clicked = Rc::new(RefCell::new(None::<Box<dyn Fn()>>));
        let on_redo_clicked = Rc::new(RefCell::new(None::<Box<dyn Fn()>>));

        {
            let cb = Rc::clone(&on_new_clicked);
            new_btn.connect_clicked(move |_| {
                if let Some(ref f) = *cb.borrow() {
                    f();
                }
            });
        }
        {
            let cb = Rc::clone(&on_open_clicked);
            open_btn.connect_clicked(move |_| {
                if let Some(ref f) = *cb.borrow() {
                    f();
                }
            });
        }
        {
            let cb = Rc::clone(&on_save_clicked);
            save_btn.connect_clicked(move |_| {
                if let Some(ref f) = *cb.borrow() {
                    f();
                }
            });
        }
        {
            let cb = Rc::clone(&on_export_clicked);
            export_btn.connect_clicked(move |_| {
                if let Some(ref f) = *cb.borrow() {
                    f();
                }
            });
        }
        {
            let cb = Rc::clone(&on_undo_clicked);
            undo_btn.connect_clicked(move |_| {
                if let Some(ref f) = *cb.borrow() {
                    f();
                }
            });
        }
        {
            let cb = Rc::clone(&on_redo_clicked);
            redo_btn.connect_clicked(move |_| {
                if let Some(ref f) = *cb.borrow() {
                    f();
                }
            });
        }

        Self {
            header_bar,
            title_label,
            zoom_label,
            undo_btn,
            redo_btn,
            on_new_clicked,
            on_open_clicked,
            on_save_clicked,
            on_export_clicked,
            on_undo_clicked,
            on_redo_clicked,
        }
    }

    pub fn set_title(&self, title: &str) {
        self.title_label.set_label(title);
    }

    pub fn set_zoom(&self, zoom_factor: f64) {
        self.zoom_label.set_label(&format!("{}%", (zoom_factor * 100.0).round() as i32));
    }

    pub fn connect_new<F: Fn() + 'static>(&self, f: F) {
        *self.on_new_clicked.borrow_mut() = Some(Box::new(f));
    }

    pub fn connect_open<F: Fn() + 'static>(&self, f: F) {
        *self.on_open_clicked.borrow_mut() = Some(Box::new(f));
    }

    pub fn connect_save<F: Fn() + 'static>(&self, f: F) {
        *self.on_save_clicked.borrow_mut() = Some(Box::new(f));
    }

    pub fn connect_export<F: Fn() + 'static>(&self, f: F) {
        *self.on_export_clicked.borrow_mut() = Some(Box::new(f));
    }

    pub fn connect_undo<F: Fn() + 'static>(&self, f: F) {
        *self.on_undo_clicked.borrow_mut() = Some(Box::new(f));
    }

    pub fn connect_redo<F: Fn() + 'static>(&self, f: F) {
        *self.on_redo_clicked.borrow_mut() = Some(Box::new(f));
    }
}
