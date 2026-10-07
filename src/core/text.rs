//! Estilo Tipográfico e Runs de Texto.
//! Traduzido de Compositor/Document/TypeTool.swift.

use crate::core::transform::Size;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TextAlignment {
    #[serde(rename = "Left")]
    Left,
    #[serde(rename = "Center")]
    Center,
    #[serde(rename = "Right")]
    Right,
}

impl Default for TextAlignment {
    fn default() -> Self {
        Self::Left
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerTextColorRun {
    pub location: usize,
    pub length: usize,
    pub red: f64,
    pub green: f64,
    pub blue: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerTextFontRun {
    pub location: usize,
    pub length: usize,
    #[serde(rename = "fontName")]
    pub font_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerTextStyle {
    #[serde(default = "default_text_content")]
    pub content: String,
    #[serde(default = "default_font_name", rename = "fontName")]
    pub font_name: String,
    #[serde(default = "default_font_size", rename = "fontSize")]
    pub font_size: f64,
    #[serde(default)]
    pub red: f64,
    #[serde(default)]
    pub green: f64,
    #[serde(default)]
    pub blue: f64,
    #[serde(default)]
    pub alignment: TextAlignment,
    #[serde(default)]
    pub tracking: f64,
    #[serde(default)]
    pub leading: f64,
    #[serde(rename = "boxSize", skip_serializing_if = "Option::is_none")]
    pub box_size: Option<Size>,
    #[serde(rename = "colorRuns", skip_serializing_if = "Option::is_none")]
    pub color_runs: Option<Vec<LayerTextColorRun>>,
    #[serde(rename = "fontRuns", skip_serializing_if = "Option::is_none")]
    pub font_runs: Option<Vec<LayerTextFontRun>>,
}

fn default_text_content() -> String { "Text".to_string() }
fn default_font_name() -> String { "Sans".to_string() }
fn default_font_size() -> f64 { 72.0 }

impl Default for LayerTextStyle {
    fn default() -> Self {
        Self {
            content: default_text_content(),
            font_name: default_font_name(),
            font_size: default_font_size(),
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alignment: TextAlignment::Left,
            tracking: 0.0,
            leading: 0.0,
            box_size: None,
            color_runs: None,
            font_runs: None,
        }
    }
}
