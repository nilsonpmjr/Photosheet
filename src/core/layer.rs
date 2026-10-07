//! Representação de Camada (Pixel, Pasta, Ajuste, Forma e Texto).
//! Traduzido de Compositor/Document/LayerGroups.swift, LayerAppearance.swift e Document.

use crate::core::adjustment::LayerAdjustment;
use crate::core::blend::LayerBlendMode;
use crate::core::effects::LayerEffects;
use crate::core::mask::LayerMask;
use crate::core::shape::LayerShapeStyle;
use crate::core::text::LayerTextStyle;
use crate::core::transform::LayerTransform;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum LayerKind {
    Pixel {
        image_file: Option<String>,
        /// Buffer RGBA8 não pré-multiplicado ou pré-multiplicado
        pixels: Option<Vec<u8>>,
        width: usize,
        height: usize,
    },
    Group,
    Adjustment(LayerAdjustment),
    Shape(LayerShapeStyle),
    Text(LayerTextStyle),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    pub id: Uuid,
    pub name: String,
    pub is_visible: bool,
    pub is_locked: bool,
    pub opacity: f64,
    pub blend_mode: LayerBlendMode,
    pub transform: LayerTransform,
    pub parent_id: Option<Uuid>,
    pub is_group: bool,
    pub mask: Option<LayerMask>,
    pub clipping_base_id: Option<Uuid>,
    pub effects: Option<LayerEffects>,
    pub kind: LayerKind,
}

impl Layer {
    pub fn new_pixel(name: String, width: usize, height: usize, transform: LayerTransform) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            is_visible: true,
            is_locked: false,
            opacity: 1.0,
            blend_mode: LayerBlendMode::Normal,
            transform,
            parent_id: None,
            is_group: false,
            mask: None,
            clipping_base_id: None,
            effects: None,
            kind: LayerKind::Pixel {
                image_file: None,
                pixels: Some(vec![0u8; width * height * 4]),
                width,
                height,
            },
        }
    }

    pub fn new_group(name: String, transform: LayerTransform) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            is_visible: true,
            is_locked: false,
            opacity: 1.0,
            blend_mode: LayerBlendMode::Normal,
            transform,
            parent_id: None,
            is_group: true,
            mask: None,
            clipping_base_id: None,
            effects: None,
            kind: LayerKind::Group,
        }
    }

    pub fn new_adjustment(name: String, adjustment: LayerAdjustment, transform: LayerTransform) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            is_visible: true,
            is_locked: false,
            opacity: 1.0,
            blend_mode: LayerBlendMode::Normal,
            transform,
            parent_id: None,
            is_group: false,
            mask: None,
            clipping_base_id: None,
            effects: None,
            kind: LayerKind::Adjustment(adjustment),
        }
    }
}
