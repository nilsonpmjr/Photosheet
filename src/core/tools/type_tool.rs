//! Mecanismo da Ferramenta de Texto (TypeTool).
//! Traduzido de Compositor/Document/TypeTool.swift e LayerText.swift.

use crate::core::session::TextToolSettings;
use crate::core::text::LayerTextStyle;
use gtk4::cairo::{Context, FontSlant, FontWeight, Format, ImageSurface};

#[derive(Debug, Clone, PartialEq)]
pub struct TextItem {
    pub text: String,
    pub font_family: String,
    pub font_size: f64,
    pub color: [u8; 4],
    pub x: f64,
    pub y: f64,
}

pub struct TypeEngine;

impl TypeEngine {
    pub fn create_text_item(x: f64, y: f64, text: String, settings: &TextToolSettings) -> TextItem {
        TextItem {
            text,
            font_family: settings.font_family.clone(),
            font_size: settings.font_size,
            color: settings.color,
            x,
            y,
        }
    }

    /// Rasteriza o fallback que acompanha o estilo editável no arquivo `.comp`.
    pub fn rasterize_text(style: &LayerTextStyle) -> Option<(Vec<u8>, usize, usize)> {
        let scratch = ImageSurface::create(Format::ARgb32, 1, 1).ok()?;
        let context = Context::new(&scratch).ok()?;
        context.select_font_face(&style.font_name, FontSlant::Normal, FontWeight::Normal);
        context.set_font_size(style.font_size.max(1.0));
        let line_height = if style.leading > 0.0 { style.leading } else { style.font_size * 1.2 };
        let width = style.content.lines()
            .map(|line| context.text_extents(line).map_or(0.0, |extents| extents.x_advance()))
            .fold(1.0_f64, f64::max)
            .ceil() as usize + 2;
        let height = ((style.content.lines().count().max(1) as f64 * line_height).ceil() as usize + 2).max(1);
        drop(context);

        let mut surface = ImageSurface::create(Format::ARgb32, width as i32, height as i32).ok()?;
        let context = Context::new(&surface).ok()?;
        context.select_font_face(&style.font_name, FontSlant::Normal, FontWeight::Normal);
        context.set_font_size(style.font_size.max(1.0));
        context.set_source_rgb(style.red, style.green, style.blue);
        for (index, line) in style.content.lines().enumerate() {
            context.move_to(1.0, style.font_size + index as f64 * line_height);
            let _ = context.show_text(line);
        }
        drop(context);
        let stride = surface.stride() as usize;
        let data = surface.data().ok()?;
        let mut pixels = vec![0; width * height * 4];
        for y in 0..height {
            for x in 0..width {
                let source = &data[y * stride + x * 4..y * stride + x * 4 + 4];
                let target = &mut pixels[(y * width + x) * 4..(y * width + x + 1) * 4];
                let alpha = source[3];
                target[0] = if alpha == 0 { 0 } else { (source[2] as u16 * 255 / alpha as u16) as u8 };
                target[1] = if alpha == 0 { 0 } else { (source[1] as u16 * 255 / alpha as u16) as u8 };
                target[2] = if alpha == 0 { 0 } else { (source[0] as u16 * 255 / alpha as u16) as u8 };
                target[3] = alpha;
            }
        }
        Some((pixels, width, height))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_item_creation() {
        let settings = TextToolSettings {
            font_family: "Cantarell".into(),
            font_size: 24.0,
            color: [255, 0, 0, 255],
        };
        let item = TypeEngine::create_text_item(100.0, 150.0, "Photosheet".into(), &settings);
        assert_eq!(item.text, "Photosheet");
        assert_eq!(item.font_size, 24.0);
    }
}
