//! Diálogo de Novo Documento (NewCanvasSheet).
//! Traduzido de Compositor/UI/NewCanvasSheet.swift e DocumentLimits.swift.

use crate::core::limits::DocumentLimits;
use gtk4::prelude::*;
use gtk4::{
    Adjustment, Box, Button, Dialog, DropDown, Entry, Label, Orientation, ResponseType, SpinButton,
    StringList, Window,
};

pub struct NewDocumentParams {
    pub width: usize,
    pub height: usize,
    pub resolution: f64,
    pub background_color: Option<[u8; 4]>,
}

pub struct NewDocumentDialog;

impl NewDocumentDialog {
    pub fn show<F: Fn(NewDocumentParams) + 'static>(parent: &impl IsA<Window>, on_confirm: F) {
        let dialog = Dialog::builder()
            .transient_for(parent)
            .modal(true)
            .title("Novo Documento")
            .use_header_bar(1)
            .default_width(400)
            .build();

        let content = dialog.content_area();
        content.set_spacing(12);
        content.set_margin_start(18);
        content.set_margin_end(18);
        content.set_margin_top(18);
        content.set_margin_bottom(18);

        // Predefinições
        let preset_box = Box::new(Orientation::Horizontal, 8);
        preset_box.append(&Label::new(Some("Predefinição:")));
        let preset_names = StringList::new(&[
            "Personalizado",
            "Full HD (1920 × 1080)",
            "4K UHD (3840 × 2160)",
            "Quadrado (1080 × 1080)",
            "A4 a 300 DPI (2480 × 3508)",
        ]);
        let preset_dropdown = DropDown::new(Some(preset_names), gtk4::Expression::NONE);
        preset_box.append(&preset_dropdown);
        content.append(&preset_box);

        // Largura
        let width_box = Box::new(Orientation::Horizontal, 8);
        width_box.append(&Label::new(Some("Largura (px):")));
        let width_adj = Adjustment::new(
            1920.0,
            1.0,
            DocumentLimits::MAX_SIDE as f64,
            1.0,
            100.0,
            0.0,
        );
        let width_spin = SpinButton::new(Some(&width_adj), 1.0, 0);
        width_box.append(&width_spin);
        content.append(&width_box);

        // Altura
        let height_box = Box::new(Orientation::Horizontal, 8);
        height_box.append(&Label::new(Some("Altura (px):")));
        let height_adj = Adjustment::new(
            1080.0,
            1.0,
            DocumentLimits::MAX_SIDE as f64,
            1.0,
            100.0,
            0.0,
        );
        let height_spin = SpinButton::new(Some(&height_adj), 1.0, 0);
        height_box.append(&height_spin);
        content.append(&height_box);

        // Resolução
        let res_box = Box::new(Orientation::Horizontal, 8);
        res_box.append(&Label::new(Some("Resolução (DPI):")));
        let res_adj = Adjustment::new(72.0, 1.0, 1200.0, 1.0, 10.0, 0.0);
        let res_spin = SpinButton::new(Some(&res_adj), 1.0, 0);
        res_box.append(&res_spin);
        content.append(&res_box);

        // Preenchimento de Fundo
        let bg_box = Box::new(Orientation::Horizontal, 8);
        bg_box.append(&Label::new(Some("Conteúdo de Fundo:")));
        let bg_names = StringList::new(&["Branco", "Transparente", "Preto"]);
        let bg_dropdown = DropDown::new(Some(bg_names), gtk4::Expression::NONE);
        bg_box.append(&bg_dropdown);
        content.append(&bg_box);

        // Atualizar dimensões ao trocar predefinição
        {
            let w_spin = width_spin.clone();
            let h_spin = height_spin.clone();
            let r_spin = res_spin.clone();
            preset_dropdown.connect_selected_notify(move |d| match d.selected() {
                1 => {
                    w_spin.set_value(1920.0);
                    h_spin.set_value(1080.0);
                    r_spin.set_value(72.0);
                }
                2 => {
                    w_spin.set_value(3840.0);
                    h_spin.set_value(2160.0);
                    r_spin.set_value(72.0);
                }
                3 => {
                    w_spin.set_value(1080.0);
                    h_spin.set_value(1080.0);
                    r_spin.set_value(72.0);
                }
                4 => {
                    w_spin.set_value(2480.0);
                    h_spin.set_value(3508.0);
                    r_spin.set_value(300.0);
                }
                _ => {}
            });
        }

        dialog.add_button("Cancelar", ResponseType::Cancel);
        let create_btn = dialog.add_button("Criar", ResponseType::Ok);
        create_btn.add_css_class("suggested-action");

        dialog.connect_response(move |d, response| {
            if response == ResponseType::Ok {
                let width = width_spin.value() as usize;
                let height = height_spin.value() as usize;
                let resolution = res_spin.value();
                let background_color = match bg_dropdown.selected() {
                    0 => Some([255, 255, 255, 255]), // Branco
                    1 => None,                       // Transparente
                    2 => Some([0, 0, 0, 255]),       // Preto
                    _ => Some([255, 255, 255, 255]),
                };
                on_confirm(NewDocumentParams {
                    width,
                    height,
                    resolution,
                    background_color,
                });
            }
            d.close();
        });

        dialog.present();
    }
}
