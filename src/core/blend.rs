//! Modos de mesclagem (Blend Modes) do Photoshop e do Compositor.
//! Traduzido de Compositor/Document/LayerAppearance.swift e Compositor/Rendering/SeparableBlend.swift.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LayerBlendMode {
    #[serde(rename = "Normal")]
    Normal,

    // Darkening
    #[serde(rename = "Darken")]
    Darken,
    #[serde(rename = "Multiply")]
    Multiply,
    #[serde(rename = "Color Burn")]
    ColorBurn,
    #[serde(rename = "Linear Burn")]
    LinearBurn,

    // Lightening
    #[serde(rename = "Lighten")]
    Lighten,
    #[serde(rename = "Screen")]
    Screen,
    #[serde(rename = "Color Dodge")]
    ColorDodge,
    #[serde(rename = "Linear Dodge (Add)")]
    LinearDodge,

    // Contrast
    #[serde(rename = "Overlay")]
    Overlay,
    #[serde(rename = "Soft Light")]
    SoftLight,
    #[serde(rename = "Hard Light")]
    HardLight,
    #[serde(rename = "Vivid Light")]
    VividLight,
    #[serde(rename = "Linear Light")]
    LinearLight,
    #[serde(rename = "Pin Light")]
    PinLight,
    #[serde(rename = "Hard Mix")]
    HardMix,

    // Comparative
    #[serde(rename = "Difference")]
    Difference,
    #[serde(rename = "Exclusion")]
    Exclusion,
    #[serde(rename = "Subtract")]
    Subtract,
    #[serde(rename = "Divide")]
    Divide,

    // Component HSL
    #[serde(rename = "Hue")]
    Hue,
    #[serde(rename = "Saturation")]
    Saturation,
    #[serde(rename = "Color")]
    Color,
    #[serde(rename = "Luminosity")]
    Luminosity,
}

impl Default for LayerBlendMode {
    fn default() -> Self {
        Self::Normal
    }
}

impl LayerBlendMode {
    /// Agrupamento canônico do Photoshop exibido no menu suspenso de camadas.
    pub fn groups() -> &'static [&'static [LayerBlendMode]] {
        &[
            &[LayerBlendMode::Normal],
            &[
                LayerBlendMode::Darken,
                LayerBlendMode::Multiply,
                LayerBlendMode::ColorBurn,
                LayerBlendMode::LinearBurn,
            ],
            &[
                LayerBlendMode::Lighten,
                LayerBlendMode::Screen,
                LayerBlendMode::ColorDodge,
                LayerBlendMode::LinearDodge,
            ],
            &[
                LayerBlendMode::Overlay,
                LayerBlendMode::SoftLight,
                LayerBlendMode::HardLight,
                LayerBlendMode::VividLight,
                LayerBlendMode::LinearLight,
                LayerBlendMode::PinLight,
                LayerBlendMode::HardMix,
            ],
            &[
                LayerBlendMode::Difference,
                LayerBlendMode::Exclusion,
                LayerBlendMode::Subtract,
                LayerBlendMode::Divide,
            ],
            &[
                LayerBlendMode::Hue,
                LayerBlendMode::Saturation,
                LayerBlendMode::Color,
                LayerBlendMode::Luminosity,
            ],
        ]
    }

    pub fn all() -> &'static [LayerBlendMode] {
        &[
            LayerBlendMode::Normal,
            LayerBlendMode::Darken,
            LayerBlendMode::Multiply,
            LayerBlendMode::ColorBurn,
            LayerBlendMode::LinearBurn,
            LayerBlendMode::Lighten,
            LayerBlendMode::Screen,
            LayerBlendMode::ColorDodge,
            LayerBlendMode::LinearDodge,
            LayerBlendMode::Overlay,
            LayerBlendMode::SoftLight,
            LayerBlendMode::HardLight,
            LayerBlendMode::VividLight,
            LayerBlendMode::LinearLight,
            LayerBlendMode::PinLight,
            LayerBlendMode::HardMix,
            LayerBlendMode::Difference,
            LayerBlendMode::Exclusion,
            LayerBlendMode::Subtract,
            LayerBlendMode::Divide,
            LayerBlendMode::Hue,
            LayerBlendMode::Saturation,
            LayerBlendMode::Color,
            LayerBlendMode::Luminosity,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Darken => "Darken",
            Self::Multiply => "Multiply",
            Self::ColorBurn => "Color Burn",
            Self::LinearBurn => "Linear Burn",
            Self::Lighten => "Lighten",
            Self::Screen => "Screen",
            Self::ColorDodge => "Color Dodge",
            Self::LinearDodge => "Linear Dodge (Add)",
            Self::Overlay => "Overlay",
            Self::SoftLight => "Soft Light",
            Self::HardLight => "Hard Light",
            Self::VividLight => "Vivid Light",
            Self::LinearLight => "Linear Light",
            Self::PinLight => "Pin Light",
            Self::HardMix => "Hard Mix",
            Self::Difference => "Difference",
            Self::Exclusion => "Exclusion",
            Self::Subtract => "Subtract",
            Self::Divide => "Divide",
            Self::Hue => "Hue",
            Self::Saturation => "Saturation",
            Self::Color => "Color",
            Self::Luminosity => "Luminosity",
        }
    }

    /// Mescla um canal em ponto flutuante [0.0, 1.0] na CPU: s (source), d (destination).
    #[inline]
    pub fn blend_channel(&self, s: f32, d: f32) -> f32 {
        match self {
            Self::Normal => s,
            Self::Darken => d.min(s),
            Self::Multiply => d * s,
            Self::ColorBurn => {
                if s <= 0.0 {
                    0.0
                } else {
                    1.0 - ((1.0 - d) / s).min(1.0)
                }
            }
            Self::LinearBurn => (d + s - 1.0).max(0.0),
            Self::Lighten => d.max(s),
            Self::Screen => d + s - d * s,
            Self::ColorDodge => {
                if s >= 1.0 {
                    1.0
                } else {
                    (d / (1.0 - s)).min(1.0)
                }
            }
            Self::LinearDodge => (d + s).min(1.0),
            Self::Overlay => {
                if d <= 0.5 {
                    2.0 * d * s
                } else {
                    1.0 - 2.0 * (1.0 - d) * (1.0 - s)
                }
            }
            Self::HardLight => {
                if s <= 0.5 {
                    2.0 * d * s
                } else {
                    1.0 - 2.0 * (1.0 - d) * (1.0 - s)
                }
            }
            Self::SoftLight => {
                if s <= 0.5 {
                    d - (1.0 - 2.0 * s) * d * (1.0 - d)
                } else {
                    let d_sqrt = d.sqrt();
                    let g = if d <= 0.25 {
                        ((16.0 * d - 12.0) * d + 4.0) * d
                    } else {
                        d_sqrt
                    };
                    d + (2.0 * s - 1.0) * (g - d)
                }
            }
            Self::VividLight => {
                if s <= 0.5 {
                    if s <= 0.0 {
                        0.0
                    } else {
                        1.0 - ((1.0 - d) / (2.0 * s)).min(1.0)
                    }
                } else {
                    let s2 = 2.0 * (s - 0.5);
                    if s2 >= 1.0 {
                        1.0
                    } else {
                        (d / (1.0 - s2)).min(1.0)
                    }
                }
            }
            Self::LinearLight => (d + 2.0 * s - 1.0).clamp(0.0, 1.0),
            Self::PinLight => {
                if s <= 0.5 {
                    d.min(2.0 * s)
                } else {
                    d.max(2.0 * (s - 0.5))
                }
            }
            Self::HardMix => {
                let vivid = Self::VividLight.blend_channel(s, d);
                if vivid < 0.5 {
                    0.0
                } else {
                    1.0
                }
            }
            Self::Difference => (d - s).abs(),
            Self::Exclusion => d + s - 2.0 * d * s,
            Self::Subtract => (d - s).max(0.0),
            Self::Divide => {
                if s <= 0.0 {
                    1.0
                } else {
                    (d / s).min(1.0)
                }
            }
            // Modos HSL são não separáveis por canal individual
            Self::Hue | Self::Saturation | Self::Color | Self::Luminosity => s,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blend_mode_serde() {
        let json = serde_json::to_string(&LayerBlendMode::ColorBurn).unwrap();
        assert_eq!(json, "\"Color Burn\"");
        let mode: LayerBlendMode = serde_json::from_str("\"Linear Dodge (Add)\"").unwrap();
        assert_eq!(mode, LayerBlendMode::LinearDodge);
    }

    #[test]
    fn test_blend_channel_math() {
        assert_eq!(LayerBlendMode::Normal.blend_channel(0.8, 0.2), 0.8);
        assert!((LayerBlendMode::Multiply.blend_channel(0.5, 0.5) - 0.25).abs() < 1e-6);
        assert!((LayerBlendMode::Screen.blend_channel(0.5, 0.5) - 0.75).abs() < 1e-6);
    }
}
