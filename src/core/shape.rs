//! Estilo e Parâmetros de Formas Vetoriais (Shape Tool).
//! Traduzido de Compositor/Document/ShapeTool.swift.

use crate::core::transform::Point;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShapeKind {
    #[serde(rename = "Rectangle")]
    Rectangle,
    #[serde(rename = "Rounded rectangle")]
    RoundedRectangle,
    #[serde(rename = "Ellipse")]
    Ellipse,
    #[serde(rename = "Line")]
    Line,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerShapeStyle {
    pub kind: ShapeKind,
    pub red: f64,
    pub green: f64,
    pub blue: f64,
    #[serde(rename = "cornerRadius")]
    pub corner_radius: f64,
    #[serde(rename = "lineWidth", skip_serializing_if = "Option::is_none")]
    pub line_width: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<Point>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<Point>,
}
