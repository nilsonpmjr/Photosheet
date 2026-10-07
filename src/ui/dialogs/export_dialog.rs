//! Diálogo de Exportação de Imagem (JPEG / PNG / TIFF).
//! Traduzido de Compositor/UI/CanvasSizeSheet.swift e Document.

use crate::core::document::Document;
use gtk4::prelude::*;
use gtk4::{
    Adjustment, Box as GtkBox, Dialog, DropDown, Entry, Label, Orientation, ResponseType, Scale,
    SpinButton, StringList, Window,
};
use std::path::PathBuf;

pub struct ExportParams {
    pub file_path: PathBuf,
    pub format: ExportFormat,
    pub quality: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Png,
    Jpeg,
    Tiff,
}

pub struct ExportDialog;

impl ExportDialog {
    pub fn show<F: Fn(ExportParams) + 'static>(
        parent: &impl IsA<Window>,
        doc: &Document,
        on_confirm: F,
    ) {
        let dialog = Dialog::builder()
            .transient_for(parent)
            .modal(true)
            .title("Exportar Imagem")
            .use_header_bar(1)
            .default_width(420)
            .build();

        let content = dialog.content_area();
        content.set_spacing(10);
        content.set_margin_start(16);
        content.set_margin_end(16);
        content.set_margin_top(16);
        content.set_margin_bottom(16);

        // Formato
        let format_box = GtkBox::new(Orientation::Horizontal, 8);
        format_box.append(&Label::new(Some("Formato:")));
        let format_names = StringList::new(&["PNG (Sem perdas)", "JPEG (Comprimido)", "TIFF"]);
        let format_dropdown = DropDown::new(Some(format_names), gtk4::Expression::NONE);
        format_box.append(&format_dropdown);
        content.append(&format_box);

        // Qualidade JPEG
        let quality_box = GtkBox::new(Orientation::Horizontal, 8);
        quality_box.append(&Label::new(Some("Qualidade (%):")));
        let quality_scale = Scale::with_range(Orientation::Horizontal, 1.0, 100.0, 1.0);
        quality_scale.set_value(90.0);
        quality_scale.set_hexpand(true);
        quality_scale.set_draw_value(true);
        quality_box.append(&quality_scale);
        content.append(&quality_box);

        // Caminho do arquivo de destino
        let path_box = GtkBox::new(Orientation::Horizontal, 8);
        path_box.append(&Label::new(Some("Salvar em:")));
        let path_entry = Entry::new();
        path_entry.set_hexpand(true);
        path_entry.set_text(&format!(
            "{}/imagem_exportada.png",
            std::env::var("HOME").unwrap_or_else(|_| ".".into())
        ));
        path_box.append(&path_entry);
        content.append(&path_box);

        // Atualizar extensão ao trocar formato
        {
            let p_entry = path_entry.clone();
            format_dropdown.connect_selected_notify(move |d| {
                let current_text = p_entry.text().to_string();
                let base = if let Some(idx) = current_text.rfind('.') {
                    &current_text[..idx]
                } else {
                    &current_text
                };
                let ext = match d.selected() {
                    0 => ".png",
                    1 => ".jpg",
                    2 => ".tiff",
                    _ => ".png",
                };
                p_entry.set_text(&format!("{}{}", base, ext));
            });
        }

        // Estimativa de Tamanho
        let size_label = Label::builder()
            .label(&format!("Resolução: {} × {} pixels", doc.width, doc.height))
            .css_classes(["dim-label"])
            .build();
        content.append(&size_label);

        dialog.add_button("Cancelar", ResponseType::Cancel);
        let export_btn = dialog.add_button("Exportar", ResponseType::Ok);
        export_btn.add_css_class("suggested-action");

        dialog.connect_response(move |d, response| {
            if response == ResponseType::Ok {
                let format = match format_dropdown.selected() {
                    0 => ExportFormat::Png,
                    1 => ExportFormat::Jpeg,
                    2 => ExportFormat::Tiff,
                    _ => ExportFormat::Png,
                };
                let quality = quality_scale.value().clamp(1.0, 100.0) as u8;
                let file_path = PathBuf::from(path_entry.text().as_str());

                on_confirm(ExportParams {
                    file_path,
                    format,
                    quality,
                });
            }
            d.close();
        });

        dialog.present();
    }

    /// Salva o buffer composto no caminho especificado
    pub fn export_composite(
        pixels: &[u8],
        width: u32,
        height: u32,
        params: &ExportParams,
    ) -> Result<(), image::ImageError> {
        match params.format {
            ExportFormat::Png => {
                image::save_buffer(
                    &params.file_path,
                    pixels,
                    width,
                    height,
                    image::ExtendedColorType::Rgba8,
                )?;
            }
            ExportFormat::Jpeg => {
                // JPEG não tem canal alfa; converte RGBA para RGB sobre fundo branco
                let mut rgb = Vec::with_capacity((width * height * 3) as usize);
                for chunk in pixels.chunks_exact(4) {
                    let a = chunk[3] as f32 / 255.0;
                    let r = (chunk[0] as f32 * a + 255.0 * (1.0 - a)).round() as u8;
                    let g = (chunk[1] as f32 * a + 255.0 * (1.0 - a)).round() as u8;
                    let b = (chunk[2] as f32 * a + 255.0 * (1.0 - a)).round() as u8;
                    rgb.push(r);
                    rgb.push(g);
                    rgb.push(b);
                }
                image::save_buffer(
                    &params.file_path,
                    &rgb,
                    width,
                    height,
                    image::ExtendedColorType::Rgb8,
                )?;
            }
            ExportFormat::Tiff => {
                image::save_buffer(
                    &params.file_path,
                    pixels,
                    width,
                    height,
                    image::ExtendedColorType::Rgba8,
                )?;
            }
        }
        Ok(())
    }
}
