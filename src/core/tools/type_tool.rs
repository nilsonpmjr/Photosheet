//! Mecanismo da Ferramenta de Texto (TypeTool).
//! Traduzido de Compositor/Document/TypeTool.swift e LayerText.swift.

use crate::core::session::TextToolSettings;

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
