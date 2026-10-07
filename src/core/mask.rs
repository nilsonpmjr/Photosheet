//! Sistema de Máscaras (Layer Masks, Clipping Masks e Group Masks).
//! Traduzido de Compositor/Document/LayerMask.swift e LiveLayerMask.swift.

use crate::core::transform::LayerTransform;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct LayerMask {
    /// Buffer de cobertura em escala de cinza de 8 bits (0 = transparente, 255 = opaco).
    pub data: Vec<u8>,
    pub width: usize,
    pub height: usize,
    pub enabled: bool,
    /// Se true, a máscara acompanha as transformações da camada. Se false, tem transformação própria.
    pub linked: bool,
    pub placement: Option<LayerTransform>,
}

impl LayerMask {
    pub fn new_uniform(width: usize, height: usize, value: u8) -> Self {
        Self {
            data: vec![value; width * height],
            width,
            height,
            enabled: true,
            linked: true,
            placement: None,
        }
    }

    pub fn invert(&mut self) {
        for b in &mut self.data {
            *b = 255 - *b;
        }
    }
}

/// Enlace de máscara de recorte (Clipping Mask).
#[derive(Debug, Clone, PartialEq)]
pub struct ClippingLink {
    /// ID da camada base que fornece o canal alfa.
    pub base_layer_id: Uuid,
}
