//! Transformação de camada (matriz, translação, escala, rotação e flips).
//! Traduzido de Compositor/Document/LayerTransform.swift.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

impl Size {
    pub const ZERO: Self = Self { width: 0.0, height: 0.0 };

    pub fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LayerSampling {
    #[serde(rename = "Nearest")]
    Nearest,
    #[serde(rename = "Smooth")]
    Smooth,
    #[serde(rename = "High quality")]
    High,
}

impl Default for LayerSampling {
    fn default() -> Self {
        Self::High
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerTransform {
    pub origin: Point,
    pub size: Size,
    #[serde(default)]
    pub rotation: f64,
    #[serde(default, rename = "flipX")]
    pub flip_x: bool,
    #[serde(default, rename = "flipY")]
    pub flip_y: bool,
    #[serde(default)]
    pub sampling: LayerSampling,
}

impl LayerTransform {
    pub fn new(origin: Point, size: Size) -> Self {
        Self {
            origin,
            size,
            rotation: 0.0,
            flip_x: false,
            flip_y: false,
            sampling: LayerSampling::High,
        }
    }

    pub fn center(&self) -> Point {
        Point {
            x: self.origin.x + self.size.width / 2.0,
            y: self.origin.y + self.size.height / 2.0,
        }
    }

    pub fn radians(&self) -> f64 {
        (self.rotation % 360.0) * std::f64::consts::PI / 180.0
    }

    pub fn is_valid(&self) -> bool {
        self.origin.x.is_finite()
            && self.origin.y.is_finite()
            && self.size.width.is_finite()
            && self.size.height.is_finite()
            && self.rotation.is_finite()
            && (1.0..=300_000.0).contains(&self.size.width)
            && (1.0..=300_000.0).contains(&self.size.height)
            && self.origin.x.abs() <= 1_000_000.0
            && self.origin.y.abs() <= 1_000_000.0
    }

    pub fn contains(&self, p: Point) -> bool {
        let center = self.center();
        let rad = self.radians();
        let cos_r = rad.cos();
        let sin_r = rad.sin();

        let dx = p.x - center.x;
        let dy = p.y - center.y;

        let local_x = dx * cos_r + dy * sin_r;
        let local_y = -dx * sin_r + dy * cos_r;

        local_x.abs() <= self.size.width / 2.0 && local_y.abs() <= self.size.height / 2.0
    }

    pub fn rounded(&self) -> Self {
        Self {
            origin: Point {
                x: self.origin.x.round(),
                y: self.origin.y.round(),
            },
            size: Size {
                width: self.size.width.round().max(1.0),
                height: self.size.height.round().max(1.0),
            },
            rotation: self.rotation.round(),
            flip_x: self.flip_x,
            flip_y: self.flip_y,
            sampling: self.sampling,
        }
    }
}
