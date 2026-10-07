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
    /// Formas e texto preservam o raster de fallback gravado pelo Compositor,
    /// além dos metadados editáveis. Isso permite abrir o documento sem perder
    /// a aparência caso a fonte ou o recurso vetorial não esteja disponível.
    Shape {
        style: LayerShapeStyle,
        image_file: Option<String>,
        pixels: Option<Vec<u8>>,
        width: usize,
        height: usize,
    },
    Text {
        style: LayerTextStyle,
        image_file: Option<String>,
        pixels: Option<Vec<u8>>,
        width: usize,
        height: usize,
    },
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
    /// Entrega os pixels que podem sofrer uma edição destrutiva. Formas e
    /// textos usam seu PNG de fallback e perdem os metadados editáveis antes
    /// de qualquer alteração de pixel, como no formato do Compositor.
    pub fn raster_pixels_mut(&mut self) -> Option<&mut Vec<u8>> {
        let raster = match &self.kind {
            LayerKind::Shape {
                image_file,
                pixels,
                width,
                height,
                ..
            }
            | LayerKind::Text {
                image_file,
                pixels,
                width,
                height,
                ..
            } => Some((image_file.clone(), pixels.clone(), *width, *height)),
            _ => None,
        };
        if let Some((image_file, pixels, width, height)) = raster {
            self.kind = LayerKind::Pixel {
                image_file,
                pixels,
                width,
                height,
            };
        }
        match &mut self.kind {
            LayerKind::Pixel { pixels: Some(pixels), .. } => Some(pixels),
            _ => None,
        }
    }

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

    pub fn new_adjustment(
        name: String,
        adjustment: LayerAdjustment,
        transform: LayerTransform,
    ) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::shape::{LayerShapeStyle, ShapeKind};
    use crate::core::transform::{Point, Size};

    #[test]
    fn destructive_pixel_edit_rasterizes_shape_metadata() {
        let mut layer = Layer::new_pixel(
            "Shape".to_string(),
            1,
            1,
            LayerTransform::new(Point::ZERO, Size::new(1.0, 1.0)),
        );
        layer.kind = LayerKind::Shape {
            style: LayerShapeStyle {
                kind: ShapeKind::Rectangle,
                red: 1.0,
                green: 0.0,
                blue: 0.0,
                corner_radius: 0.0,
                line_width: None,
                start: None,
                end: None,
            },
            image_file: Some("fallback.png".to_string()),
            pixels: Some(vec![255, 0, 0, 255]),
            width: 1,
            height: 1,
        };

        layer.raster_pixels_mut().unwrap()[0] = 0;

        assert!(matches!(layer.kind, LayerKind::Pixel { .. }));
        let LayerKind::Pixel { image_file, pixels, .. } = layer.kind else { unreachable!() };
        assert_eq!(image_file.as_deref(), Some("fallback.png"));
        assert_eq!(pixels.unwrap()[0], 0);
    }
}
