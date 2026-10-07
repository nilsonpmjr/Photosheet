use gtk4::prelude::*;
use gtk4::{
    Adjustment, Box as GtkBox, CheckButton, Dialog, DropDown, Label, Orientation, ResponseType, Scale,
    SpinButton, StringList, Window,
};

pub struct FilterDialogs;

impl FilterDialogs {
    /// Diálogo de Desfoque Gaussiano (Gaussian Blur)
    pub fn show_gaussian_blur<F: Fn(f64) + 'static>(parent: &impl IsA<Window>, on_confirm: F) {
        let dialog = Dialog::builder()
            .transient_for(parent)
            .modal(true)
            .title("Desfoque Gaussiano")
            .use_header_bar(1)
            .default_width(360)
            .build();

        let content = dialog.content_area();
        content.set_spacing(10);
        content.set_margin_start(16);
        content.set_margin_end(16);
        content.set_margin_top(16);
        content.set_margin_bottom(16);

        let row = GtkBox::new(Orientation::Horizontal, 8);
        row.append(&Label::new(Some("Raio (pixels):")));
        let adj = Adjustment::new(10.0, 0.1, 250.0, 0.5, 5.0, 0.0);
        let spin = SpinButton::new(Some(&adj), 0.5, 1);
        row.append(&spin);
        content.append(&row);

        dialog.add_button("Cancelar", ResponseType::Cancel);
        let ok_btn = dialog.add_button("Aplicar", ResponseType::Ok);
        ok_btn.add_css_class("suggested-action");

        dialog.connect_response(move |d, response| {
            if response == ResponseType::Ok {
                on_confirm(spin.value());
            }
            d.close();
        });

        dialog.present();
    }

    /// Diálogo de Ruído (Add Noise)
    pub fn show_add_noise<F: Fn(f64, bool) + 'static>(parent: &impl IsA<Window>, on_confirm: F) {
        let dialog = Dialog::builder()
            .transient_for(parent)
            .modal(true)
            .title("Adicionar Ruído")
            .use_header_bar(1)
            .default_width(360)
            .build();

        let content = dialog.content_area();
        content.set_spacing(10);
        content.set_margin_start(16);
        content.set_margin_end(16);
        content.set_margin_top(16);
        content.set_margin_bottom(16);

        let row = GtkBox::new(Orientation::Horizontal, 8);
        row.append(&Label::new(Some("Quantidade (%):")));
        let scale = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
        scale.set_value(25.0);
        scale.set_hexpand(true);
        scale.set_draw_value(true);
        row.append(&scale);
        content.append(&row);

        let mono_check = CheckButton::with_label("Monocromático");
        mono_check.set_active(true);
        content.append(&mono_check);

        dialog.add_button("Cancelar", ResponseType::Cancel);
        let ok_btn = dialog.add_button("Aplicar", ResponseType::Ok);
        ok_btn.add_css_class("suggested-action");

        dialog.connect_response(move |d, response| {
            if response == ResponseType::Ok {
                on_confirm(scale.value(), mono_check.is_active());
            }
            d.close();
        });

        dialog.present();
    }

    /// Diálogo de Distorção de Lente (Lens Distortion)
    pub fn show_lens_distortion<F: Fn(f64) + 'static>(parent: &impl IsA<Window>, on_confirm: F) {
        let dialog = Dialog::builder()
            .transient_for(parent)
            .modal(true)
            .title("Correção de Lente / Distorção")
            .use_header_bar(1)
            .default_width(360)
            .build();

        let content = dialog.content_area();
        content.set_spacing(10);
        content.set_margin_start(16);
        content.set_margin_end(16);
        content.set_margin_top(16);
        content.set_margin_bottom(16);

        let row = GtkBox::new(Orientation::Horizontal, 8);
        row.append(&Label::new(Some("Distorção:")));
        let scale = Scale::with_range(Orientation::Horizontal, -100.0, 100.0, 1.0);
        scale.set_value(0.0);
        scale.set_hexpand(true);
        scale.set_draw_value(true);
        row.append(&scale);
        content.append(&row);

        dialog.add_button("Cancelar", ResponseType::Cancel);
        let ok_btn = dialog.add_button("Aplicar", ResponseType::Ok);
        ok_btn.add_css_class("suggested-action");

        dialog.connect_response(move |d, response| {
            if response == ResponseType::Ok {
                on_confirm(scale.value() / 100.0);
            }
            d.close();
        });

        dialog.present();
    }

    /// Diálogo de Difusão / Dither
    pub fn show_dither<F: Fn(i32) + 'static>(parent: &impl IsA<Window>, on_confirm: F) {
        let dialog = Dialog::builder()
            .transient_for(parent)
            .modal(true)
            .title("Dither / Halftone")
            .use_header_bar(1)
            .default_width(360)
            .build();

        let content = dialog.content_area();
        content.set_spacing(10);
        content.set_margin_start(16);
        content.set_margin_end(16);
        content.set_margin_top(16);
        content.set_margin_bottom(16);

        let row = GtkBox::new(Orientation::Horizontal, 8);
        row.append(&Label::new(Some("Algoritmo:")));
        let names = StringList::new(&[
            "Bayer 2x2",
            "Bayer 4x4",
            "Bayer 8x8",
            "Blue Noise",
            "Floyd-Steinberg",
        ]);
        let dropdown = DropDown::new(Some(names), gtk4::Expression::NONE);
        row.append(&dropdown);
        content.append(&row);

        dialog.add_button("Cancelar", ResponseType::Cancel);
        let ok_btn = dialog.add_button("Aplicar", ResponseType::Ok);
        ok_btn.add_css_class("suggested-action");

        dialog.connect_response(move |d, response| {
            if response == ResponseType::Ok {
                on_confirm(dropdown.selected() as i32);
            }
            d.close();
        });

        dialog.present();
    }
}
