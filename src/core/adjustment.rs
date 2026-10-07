//! Configurações de Camadas de Ajuste (Adjustment Layers).
//! Traduzido de Compositor/Document/LayerAdjustment.swift.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AdjustmentKind {
    #[serde(rename = "Hue/Saturation")]
    HueSaturation,
    #[serde(rename = "Levels")]
    Levels,
    #[serde(rename = "Curves")]
    Curves,
    #[serde(rename = "Exposure")]
    Exposure,
    #[serde(rename = "Gradient Map")]
    GradientMap,
    #[serde(rename = "Grain")]
    Grain,
    #[serde(rename = "Add Noise")]
    AddNoise,
    #[serde(rename = "Gaussian Blur")]
    GaussianBlur,
    #[serde(rename = "Motion Blur")]
    MotionBlur,
    #[serde(rename = "Invert")]
    Invert,
    #[serde(rename = "Black & White")]
    BlackWhite,
    #[serde(rename = "Color Balance")]
    ColorBalance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HueSaturationSettings {
    #[serde(default)]
    pub hue: f64,
    #[serde(default)]
    pub saturation: f64,
    #[serde(default)]
    pub lightness: f64,
    #[serde(default)]
    pub colorize: bool,
}

impl Default for HueSaturationSettings {
    fn default() -> Self {
        Self {
            hue: 0.0,
            saturation: 0.0,
            lightness: 0.0,
            colorize: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LevelsChannel {
    #[serde(default = "default_zero")]
    pub in_black: f64,
    #[serde(default = "default_one")]
    pub in_gamma: f64,
    #[serde(default = "default_255")]
    pub in_white: f64,
    #[serde(default = "default_zero")]
    pub out_black: f64,
    #[serde(default = "default_255")]
    pub out_white: f64,
}

fn default_zero() -> f64 { 0.0 }
fn default_one() -> f64 { 1.0 }
fn default_255() -> f64 { 255.0 }

impl Default for LevelsChannel {
    fn default() -> Self {
        Self {
            in_black: 0.0,
            in_gamma: 1.0,
            in_white: 255.0,
            out_black: 0.0,
            out_white: 255.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct LevelsSettings {
    #[serde(default)]
    pub rgb: LevelsChannel,
    #[serde(default)]
    pub red: LevelsChannel,
    #[serde(default)]
    pub green: LevelsChannel,
    #[serde(default)]
    pub blue: LevelsChannel,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CurvePoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CurvesSettings {
    #[serde(default = "default_curve")]
    pub rgb: Vec<CurvePoint>,
    #[serde(default = "default_curve")]
    pub red: Vec<CurvePoint>,
    #[serde(default = "default_curve")]
    pub green: Vec<CurvePoint>,
    #[serde(default = "default_curve")]
    pub blue: Vec<CurvePoint>,
}

fn default_curve() -> Vec<CurvePoint> {
    vec![CurvePoint { x: 0.0, y: 0.0 }, CurvePoint { x: 1.0, y: 1.0 }]
}

impl Default for CurvesSettings {
    fn default() -> Self {
        Self {
            rgb: default_curve(),
            red: default_curve(),
            green: default_curve(),
            blue: default_curve(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ExposureSettings {
    #[serde(default)]
    pub exposure: f64,
    #[serde(default)]
    pub offset: f64,
    #[serde(default = "default_one")]
    pub gamma: f64,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct GradientMapSettings {
    #[serde(default)]
    pub reversed: bool,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct GrainSettings {
    #[serde(default)]
    pub amount: f64,
    #[serde(default)]
    pub roughness: f64,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct BlackWhiteSettings {
    #[serde(default)]
    pub reds: f64,
    #[serde(default)]
    pub yellows: f64,
    #[serde(default)]
    pub greens: f64,
    #[serde(default)]
    pub cyans: f64,
    #[serde(default)]
    pub blues: f64,
    #[serde(default)]
    pub magentas: f64,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ColorBalanceSettings {
    #[serde(default)]
    pub preserve_luminosity: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerAdjustment {
    pub kind: AdjustmentKind,
    #[serde(default)]
    pub hue: f64,
    #[serde(default)]
    pub saturation: f64,
    #[serde(default)]
    pub lightness: f64,
    #[serde(default)]
    pub colorize: bool,
    #[serde(rename = "hsvSettings", skip_serializing_if = "Option::is_none")]
    pub hsv_settings: Option<HueSaturationSettings>,
    #[serde(default)]
    pub levels: LevelsSettings,
    #[serde(default)]
    pub curves: CurvesSettings,
    #[serde(rename = "exposureSettings", skip_serializing_if = "Option::is_none")]
    pub exposure_settings: Option<ExposureSettings>,
    #[serde(rename = "gradientMapSettings", skip_serializing_if = "Option::is_none")]
    pub gradient_map_settings: Option<GradientMapSettings>,
    #[serde(rename = "grainSettings", skip_serializing_if = "Option::is_none")]
    pub grain_settings: Option<GrainSettings>,
    #[serde(rename = "blackWhiteSettings", skip_serializing_if = "Option::is_none")]
    pub black_white_settings: Option<BlackWhiteSettings>,
    #[serde(rename = "colorBalanceSettings", skip_serializing_if = "Option::is_none")]
    pub color_balance_settings: Option<ColorBalanceSettings>,
    #[serde(rename = "blurRadius", skip_serializing_if = "Option::is_none")]
    pub blur_radius: Option<f64>,
    #[serde(rename = "motionAngle", skip_serializing_if = "Option::is_none")]
    pub motion_angle: Option<f64>,
    #[serde(rename = "motionDistance", skip_serializing_if = "Option::is_none")]
    pub motion_distance: Option<f64>,
    #[serde(rename = "noiseAmount", skip_serializing_if = "Option::is_none")]
    pub noise_amount: Option<f64>,
    #[serde(rename = "noiseGaussian", skip_serializing_if = "Option::is_none")]
    pub noise_gaussian: Option<bool>,
    #[serde(rename = "noiseMonochromatic", skip_serializing_if = "Option::is_none")]
    pub noise_monochromatic: Option<bool>,
    #[serde(rename = "noiseSeed", skip_serializing_if = "Option::is_none")]
    pub noise_seed: Option<u32>,
}

impl LayerAdjustment {
    pub fn new(kind: AdjustmentKind) -> Self {
        Self {
            kind,
            hue: 0.0,
            saturation: 0.0,
            lightness: 0.0,
            colorize: false,
            hsv_settings: None,
            levels: LevelsSettings::default(),
            curves: CurvesSettings::default(),
            exposure_settings: None,
            gradient_map_settings: None,
            grain_settings: None,
            black_white_settings: None,
            color_balance_settings: None,
            blur_radius: None,
            motion_angle: None,
            motion_distance: None,
            noise_amount: None,
            noise_gaussian: None,
            noise_monochromatic: None,
            noise_seed: None,
        }
    }

    pub fn levels(in_black: f64, in_gamma: f64, in_white: f64, out_black: f64, out_white: f64) -> Self {
        let mut adj = Self::new(AdjustmentKind::Levels);
        adj.levels.rgb = LevelsChannel {
            in_black,
            in_gamma,
            in_white,
            out_black,
            out_white,
        };
        adj
    }

    pub fn hue_saturation(hue: f64, saturation: f64, lightness: f64) -> Self {
        let mut adj = Self::new(AdjustmentKind::HueSaturation);
        adj.hue = hue;
        adj.saturation = saturation;
        adj.lightness = lightness;
        adj
    }

    pub fn exposure(exposure: f64, offset: f64, gamma: f64) -> Self {
        let mut adj = Self::new(AdjustmentKind::Exposure);
        adj.exposure_settings = Some(ExposureSettings { exposure, offset, gamma });
        adj
    }

    pub fn curves() -> Self {
        Self::new(AdjustmentKind::Curves)
    }

    pub fn invert() -> Self {
        Self::new(AdjustmentKind::Invert)
    }

    pub fn black_and_white() -> Self {
        Self::new(AdjustmentKind::BlackWhite)
    }

    pub fn gaussian_blur(radius: f64) -> Self {
        let mut adj = Self::new(AdjustmentKind::GaussianBlur);
        adj.blur_radius = Some(radius);
        adj
    }

    pub fn add_noise(amount: f64, gaussian: bool, mono: bool) -> Self {
        let mut adj = Self::new(AdjustmentKind::AddNoise);
        adj.noise_amount = Some(amount);
        adj.noise_gaussian = Some(gaussian);
        adj.noise_monochromatic = Some(mono);
        adj
    }
}

