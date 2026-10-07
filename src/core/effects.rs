//! Efeitos e Estilos de Camada (Layer Effects / Styles).
//! Traduzido de Compositor/Document/LayerEffects.swift.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrokeEffect {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default = "default_stroke_size")]
    pub size: f64,
    #[serde(default)]
    pub red: f64,
    #[serde(default)]
    pub green: f64,
    #[serde(default)]
    pub blue: f64,
    #[serde(default = "default_one")]
    pub opacity: f64,
    #[serde(default)]
    pub inside: bool,
}

fn default_stroke_size() -> f64 { 4.0 }
fn default_one() -> f64 { 1.0 }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShadowEffect {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default = "default_angle")]
    pub angle: f64,
    #[serde(default = "default_distance")]
    pub distance: f64,
    #[serde(default = "default_blur")]
    pub blur: f64,
    #[serde(default)]
    pub red: f64,
    #[serde(default)]
    pub green: f64,
    #[serde(default)]
    pub blue: f64,
    #[serde(default = "default_half")]
    pub opacity: f64,
}

fn default_angle() -> f64 { 90.0 }
fn default_distance() -> f64 { 20.0 }
fn default_blur() -> f64 { 20.0 }
fn default_half() -> f64 { 0.5 }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorOverlayEffect {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub red: f64,
    #[serde(default)]
    pub green: f64,
    #[serde(default)]
    pub blue: f64,
    #[serde(default = "default_one")]
    pub opacity: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlowEffect {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default = "default_glow_size")]
    pub size: f64,
    #[serde(default)]
    pub red: f64,
    #[serde(default)]
    pub green: f64,
    #[serde(default)]
    pub blue: f64,
    #[serde(default = "default_half")]
    pub opacity: f64,
}

fn default_glow_size() -> f64 { 20.0 }

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct LayerEffects {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<StrokeEffect>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shadow: Option<ShadowEffect>,
    #[serde(rename = "colorOverlay", skip_serializing_if = "Option::is_none")]
    pub color_overlay: Option<ColorOverlayEffect>,
    #[serde(rename = "innerShadow", skip_serializing_if = "Option::is_none")]
    pub inner_shadow: Option<ShadowEffect>,
    #[serde(rename = "outerGlow", skip_serializing_if = "Option::is_none")]
    pub outer_glow: Option<GlowEffect>,
    #[serde(rename = "innerGlow", skip_serializing_if = "Option::is_none")]
    pub inner_glow: Option<GlowEffect>,
}
