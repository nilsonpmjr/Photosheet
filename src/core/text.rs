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

fn default_text_content() -> String {
    "Text".to_string()
}
fn default_font_name() -> String {
    "Sans".to_string()
}
fn default_font_size() -> f64 {
    72.0
}

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

impl LayerTextStyle {
    /// Mirrors Compositor's on-disk text invariants. Ranges use UTF-16
    /// offsets because that is what the macOS editor serializes.
    pub fn is_valid(&self) -> bool {
        self.content.encode_utf16().count() <= 100_000
            && valid_font_name(&self.font_name)
            && self.font_size.is_finite()
            && (1.0..=2_000.0).contains(&self.font_size)
            && valid_color(self.red, self.green, self.blue)
            && self.tracking.is_finite()
            && (-100.0..=1_000.0).contains(&self.tracking)
            && self.leading.is_finite()
            && (0.0..=5_000.0).contains(&self.leading)
            && self.box_size.is_none_or(|size| {
                size.width.is_finite()
                    && size.height.is_finite()
                    && (16.0..=30_000.0).contains(&size.width)
                    && (16.0..=30_000.0).contains(&size.height)
                    && size.width * size.height <= 100_000_000.0
            })
            && self.color_runs.as_ref().is_none_or(|runs| {
                valid_runs(runs, self.content.encode_utf16().count(), |run| {
                    valid_color(run.red, run.green, run.blue)
                })
            })
            && self.font_runs.as_ref().is_none_or(|runs| {
                valid_runs(runs, self.content.encode_utf16().count(), |run| {
                    valid_font_name(&run.font_name)
                })
            })
    }
}

fn valid_color(red: f64, green: f64, blue: f64) -> bool {
    [red, green, blue]
        .into_iter()
        .all(|component| component.is_finite() && (0.0..=1.0).contains(&component))
}

fn valid_font_name(name: &str) -> bool {
    !name.is_empty() && name.len() <= 200 && !name.chars().any(char::is_control)
}

fn valid_runs<T>(runs: &[T], content_len: usize, valid: impl Fn(&T) -> bool) -> bool
where
    T: TextRun,
{
    let mut end = 0;
    !runs.is_empty()
        && runs.iter().all(|run| {
            let start = run.location();
            let Some(next_end) = start.checked_add(run.length()) else {
                return false;
            };
            let is_valid =
                start >= end && run.length() > 0 && next_end <= content_len && valid(run);
            end = next_end;
            is_valid
        })
}

trait TextRun {
    fn location(&self) -> usize;
    fn length(&self) -> usize;
}

impl TextRun for LayerTextColorRun {
    fn location(&self) -> usize {
        self.location
    }
    fn length(&self) -> usize {
        self.length
    }
}

impl TextRun for LayerTextFontRun {
    fn location(&self) -> usize {
        self.location
    }
    fn length(&self) -> usize {
        self.length
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_utf16_text_runs_without_overlap() {
        let mut style = LayerTextStyle {
            content: "A😀B".to_string(),
            color_runs: Some(vec![LayerTextColorRun {
                location: 1,
                length: 2,
                red: 1.0,
                green: 0.0,
                blue: 0.0,
            }]),
            ..Default::default()
        };
        assert!(style.is_valid());

        style.color_runs.as_mut().unwrap()[0].length = 4;
        assert!(!style.is_valid());
    }
}
