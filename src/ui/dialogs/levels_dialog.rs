//! Diálogo Interativo de Níveis (LevelsDialog).
//! Traduzido de Compositor/UI/LevelsSheet.swift e LevelsPixels.c.

use crate::core::adjustment::LayerAdjustment;
use gtk4::prelude::*;
use gtk4::{Adjustment, Box, Dialog, Label, Orientation, ResponseType, Scale, Window};

pub struct LevelsDialog;

impl LevelsDialog {
    pub fn show<F: Fn(LayerAdjustment) + 'static>(parent: &impl IsA<Window>, on_confirm: F) {
        let dialog = Dialog::builder()
            .transient_for(parent)
            .modal(true)
            .title("Ajuste de Níveis")
            .use_header_bar(1)
            .default_width(450)
            .build();

        let content = dialog.content_area();
        content.set_spacing(10);
        content.set_margin_start(16);
        content.set_margin_end(16);
        content.set_margin_top(16);
        content.set_margin_bottom(16);

        // Níveis de Entrada
        let in_label = Label::builder()
            .label("Níveis de Entrada:")
            .halign(gtk4::Align::Start)
            .css_classes(["heading"])
            .build();
        content.append(&in_label);

        // Preto Entrada
        let in_black_box = Box::new(Orientation::Horizontal, 8);
        in_black_box.append(&Label::new(Some("Preto:")));
        let in_black_scale = Scale::with_range(Orientation::Horizontal, 0.0, 255.0, 1.0);
        in_black_scale.set_value(0.0);
        in_black_scale.set_hexpand(true);
        in_black_scale.set_draw_value(true);
        in_black_box.append(&in_black_scale);
        content.append(&in_black_box);

        // Meios-tons (Gamma)
        let in_gamma_box = Box::new(Orientation::Horizontal, 8);
        in_gamma_box.append(&Label::new(Some("Gama:")));
        let in_gamma_scale = Scale::with_range(Orientation::Horizontal, 0.1, 9.99, 0.05);
        in_gamma_scale.set_value(1.0);
        in_gamma_scale.set_hexpand(true);
        in_gamma_scale.set_draw_value(true);
        in_gamma_box.append(&in_gamma_scale);
        content.append(&in_gamma_box);

        // Branco Entrada
        let in_white_box = Box::new(Orientation::Horizontal, 8);
        in_white_box.append(&Label::new(Some("Branco:")));
        let in_white_scale = Scale::with_range(Orientation::Horizontal, 0.0, 255.0, 1.0);
        in_white_scale.set_value(255.0);
        in_white_scale.set_hexpand(true);
        in_white_scale.set_draw_value(true);
        in_white_box.append(&in_white_scale);
        content.append(&in_white_box);

        // Níveis de Saída
        let out_label = Label::builder()
            .label("Níveis de Saída:")
            .halign(gtk4::Align::Start)
            .css_classes(["heading"])
            .build();
        content.append(&out_label);

        // Preto Saída
        let out_black_box = Box::new(Orientation::Horizontal, 8);
        out_black_box.append(&Label::new(Some("Preto:")));
        let out_black_scale = Scale::with_range(Orientation::Horizontal, 0.0, 255.0, 1.0);
        out_black_scale.set_value(0.0);
        out_black_scale.set_hexpand(true);
        out_black_scale.set_draw_value(true);
        out_black_box.append(&out_black_scale);
        content.append(&out_black_box);

        // Branco Saída
        let out_white_box = Box::new(Orientation::Horizontal, 8);
        out_white_box.append(&Label::new(Some("Branco:")));
        let out_white_scale = Scale::with_range(Orientation::Horizontal, 0.0, 255.0, 1.0);
        out_white_scale.set_value(255.0);
        out_white_scale.set_hexpand(true);
        out_white_scale.set_draw_value(true);
        out_white_box.append(&out_white_scale);
        content.append(&out_white_box);

        dialog.add_button("Cancelar", ResponseType::Cancel);
        let ok_btn = dialog.add_button("Aplicar", ResponseType::Ok);
        ok_btn.add_css_class("suggested-action");

        dialog.connect_response(move |d, response| {
            if response == ResponseType::Ok {
                let adj = LayerAdjustment::levels(
                    in_black_scale.value(),
                    in_gamma_scale.value(),
                    in_white_scale.value(),
                    out_black_scale.value(),
                    out_white_scale.value(),
                );
                on_confirm(adj);
            }
            d.close();
        });

        dialog.present();
    }
}
