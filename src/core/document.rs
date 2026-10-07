//! Modelo de Documento em Memória.
//! Traduzido de Compositor/Document/ProjectWorkspace.swift e Document.

use crate::core::layer::{Layer, LayerKind};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Erros de integridade da árvore de camadas. São independentes do formato
/// `.comp` para que a UI não consiga criar um estado que o salvamento rejeite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerGraphError {
    DuplicateLayerId,
    MissingLayer,
    ParentIsNotGroup,
    GroupKindMismatch,
    GroupCycleOrDepth,
    ClippingCycleOrDepth,
    InvalidClippingSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GuideAxis {
    #[serde(rename = "horizontal")]
    Horizontal,
    #[serde(rename = "vertical")]
    Vertical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasGuide {
    pub id: Uuid,
    pub axis: GuideAxis,
    pub position: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub id: Uuid,
    pub width: usize,
    pub height: usize,
    pub resolution: f64,
    pub active_layer_id: Option<Uuid>,
    /// Camadas em ordem bottom-to-top (o índice 0 é o fundo do documento)
    pub layers: Vec<Layer>,
    pub guides: Vec<CanvasGuide>,
}

impl Document {
    pub fn new(width: usize, height: usize, resolution: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            width,
            height,
            resolution: if resolution <= 0.0 { 72.0 } else { resolution },
            active_layer_id: None,
            layers: Vec::new(),
            guides: Vec::new(),
        }
    }

    pub fn active_layer(&self) -> Option<&Layer> {
        let id = self.active_layer_id?;
        self.layers.iter().find(|l| l.id == id)
    }

    pub fn active_layer_mut(&mut self) -> Option<&mut Layer> {
        let id = self.active_layer_id?;
        self.layers.iter_mut().find(|l| l.id == id)
    }

    pub fn find_layer(&self, id: Uuid) -> Option<&Layer> {
        self.layers.iter().find(|l| l.id == id)
    }

    pub fn find_layer_mut(&mut self, id: Uuid) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|l| l.id == id)
    }

    /// Valida as relações entre camadas já materializadas no documento.
    ///
    /// O limite de grupos acompanha o leitor de `.comp`; o limite maior para
    /// clipping evita recursão ilimitada no compositor.
    pub fn validate_layer_graph(&self) -> Result<(), LayerGraphError> {
        let by_id: HashMap<Uuid, &Layer> =
            self.layers.iter().map(|layer| (layer.id, layer)).collect();
        if by_id.len() != self.layers.len() {
            return Err(LayerGraphError::DuplicateLayerId);
        }

        for layer in &self.layers {
            if layer.is_group != matches!(layer.kind, LayerKind::Group) {
                return Err(LayerGraphError::GroupKindMismatch);
            }
            let mut seen = HashSet::from([layer.id]);
            let mut parent = layer.parent_id;
            while let Some(id) = parent {
                let Some(parent_layer) = by_id.get(&id) else {
                    return Err(LayerGraphError::MissingLayer);
                };
                if seen.len() > 64 || !seen.insert(id) {
                    return Err(LayerGraphError::GroupCycleOrDepth);
                }
                if !parent_layer.is_group {
                    return Err(LayerGraphError::ParentIsNotGroup);
                }
                parent = parent_layer.parent_id;
            }

            let mut clipping_path = HashSet::new();
            let mut current = Some(layer.id);
            while let Some(id) = current {
                if clipping_path.len() >= 256 || !clipping_path.insert(id) {
                    return Err(LayerGraphError::ClippingCycleOrDepth);
                }
                let Some(node) = by_id.get(&id) else {
                    return Err(LayerGraphError::MissingLayer);
                };
                if let Some(base_id) = node.clipping_base_id {
                    let Some(base) = by_id.get(&base_id) else {
                        return Err(LayerGraphError::MissingLayer);
                    };
                    if node.is_group
                        || base.is_group
                        || matches!(base.kind, LayerKind::Adjustment(_))
                    {
                        return Err(LayerGraphError::InvalidClippingSource);
                    }
                }
                current = node.clipping_base_id;
            }
        }
        Ok(())
    }

    /// Adiciona uma camada somente se a árvore resultante continuar válida.
    pub fn try_add_layer(&mut self, layer: Layer) -> Result<(), LayerGraphError> {
        let id = layer.id;
        self.layers.push(layer);
        if let Err(error) = self.validate_layer_graph() {
            self.layers.pop();
            return Err(error);
        }
        self.active_layer_id = Some(id);
        Ok(())
    }

    /// API compatível para os fluxos legados; estados inválidos não entram no documento.
    pub fn add_layer(&mut self, layer: Layer) {
        let _ = self.try_add_layer(layer);
    }

    pub fn try_set_parent(
        &mut self,
        id: Uuid,
        parent_id: Option<Uuid>,
    ) -> Result<(), LayerGraphError> {
        let layer = self
            .find_layer_mut(id)
            .ok_or(LayerGraphError::MissingLayer)?;
        let previous = layer.parent_id;
        layer.parent_id = parent_id;
        if let Err(error) = self.validate_layer_graph() {
            self.find_layer_mut(id)
                .expect("layer was checked above")
                .parent_id = previous;
            return Err(error);
        }
        Ok(())
    }

    pub fn try_set_clipping_base(
        &mut self,
        id: Uuid,
        clipping_base_id: Option<Uuid>,
    ) -> Result<(), LayerGraphError> {
        let layer = self
            .find_layer_mut(id)
            .ok_or(LayerGraphError::MissingLayer)?;
        let previous = layer.clipping_base_id;
        layer.clipping_base_id = clipping_base_id;
        if let Err(error) = self.validate_layer_graph() {
            self.find_layer_mut(id)
                .expect("layer was checked above")
                .clipping_base_id = previous;
            return Err(error);
        }
        Ok(())
    }

    pub fn remove_layer(&mut self, id: Uuid) -> Option<Layer> {
        if let Some(pos) = self.layers.iter().position(|l| l.id == id) {
            let removed = self.layers.remove(pos);
            for layer in &mut self.layers {
                if layer.parent_id == Some(id) {
                    layer.parent_id = removed.parent_id;
                }
                if layer.clipping_base_id == Some(id) {
                    layer.clipping_base_id = None;
                }
            }
            if self.active_layer_id == Some(id) {
                self.active_layer_id = if pos > 0 {
                    self.layers.get(pos - 1).map(|l| l.id)
                } else {
                    self.layers.first().map(|l| l.id)
                };
            }
            Some(removed)
        } else {
            None
        }
    }

    pub fn duplicate_layer(&mut self, id: Uuid) -> Option<Uuid> {
        let pos = self.layers.iter().position(|l| l.id == id)?;
        let mut cloned = self.layers[pos].clone();
        cloned.id = Uuid::new_v4();
        cloned.name = format!("{} (Cópia)", cloned.name);
        let new_id = cloned.id;
        self.layers.insert(pos + 1, cloned);
        self.active_layer_id = Some(new_id);
        Some(new_id)
    }

    pub fn reorder_layer(&mut self, id: Uuid, target_index: usize) -> bool {
        if let Some(pos) = self.layers.iter().position(|l| l.id == id) {
            let layer = self.layers.remove(pos);
            let bounded_idx = target_index.min(self.layers.len());
            self.layers.insert(bounded_idx, layer);
            true
        } else {
            false
        }
    }

    pub fn flip_layer_horizontal(&mut self, id: Uuid) {
        if let Some(l) = self.find_layer_mut(id) {
            l.transform.flip_x = !l.transform.flip_x;
            if let LayerKind::Pixel {
                ref mut pixels,
                width,
                height,
                ..
            } = l.kind
            {
                if let Some(px) = pixels {
                    let w = width;
                    let h = height;
                    for y in 0..h {
                        for x in 0..(w / 2) {
                            let left = (y * w + x) * 4;
                            let right = (y * w + (w - 1 - x)) * 4;
                            for c in 0..4 {
                                px.swap(left + c, right + c);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn flip_layer_vertical(&mut self, id: Uuid) {
        if let Some(l) = self.find_layer_mut(id) {
            l.transform.flip_y = !l.transform.flip_y;
            if let LayerKind::Pixel {
                ref mut pixels,
                width,
                height,
                ..
            } = l.kind
            {
                if let Some(px) = pixels {
                    let w = width;
                    let h = height;
                    let row_bytes = w * 4;
                    for y in 0..(h / 2) {
                        let top = y * row_bytes;
                        let bottom = (h - 1 - y) * row_bytes;
                        for b in 0..row_bytes {
                            px.swap(top + b, bottom + b);
                        }
                    }
                }
            }
        }
    }

    pub fn flip_canvas_horizontal(&mut self) {
        let w = self.width as f64;
        for l in &mut self.layers {
            l.transform.flip_x = !l.transform.flip_x;
            l.transform.origin.x = w - (l.transform.origin.x + l.transform.size.width);
        }
        for g in &mut self.guides {
            if g.axis == GuideAxis::Vertical {
                g.position = w - g.position;
            }
        }
    }

    pub fn flip_canvas_vertical(&mut self) {
        let h = self.height as f64;
        for l in &mut self.layers {
            l.transform.flip_y = !l.transform.flip_y;
            l.transform.origin.y = h - (l.transform.origin.y + l.transform.size.height);
        }
        for g in &mut self.guides {
            if g.axis == GuideAxis::Horizontal {
                g.position = h - g.position;
            }
        }
    }

    pub fn resize_canvas(
        &mut self,
        new_width: usize,
        new_height: usize,
        offset_x: f64,
        offset_y: f64,
    ) {
        self.width = new_width;
        self.height = new_height;
        for l in &mut self.layers {
            l.transform.origin.x += offset_x;
            l.transform.origin.y += offset_y;
        }
        for g in &mut self.guides {
            match g.axis {
                GuideAxis::Horizontal => g.position += offset_y,
                GuideAxis::Vertical => g.position += offset_x,
            }
        }
    }

    pub fn add_guide(&mut self, axis: GuideAxis, position: f64) -> CanvasGuide {
        let guide = CanvasGuide {
            id: Uuid::new_v4(),
            axis,
            position,
        };
        self.guides.push(guide.clone());
        guide
    }

    pub fn clear_guides(&mut self) {
        self.guides.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::transform::{LayerTransform, Point, Size};

    #[test]
    fn test_document_layer_operations() {
        let mut doc = Document::new(800, 600, 72.0);
        let l1 = Layer::new_pixel(
            "Camada 1".to_string(),
            100,
            100,
            LayerTransform::new(Point::new(0.0, 0.0), Size::new(100.0, 100.0)),
        );
        let l1_id = l1.id;
        doc.add_layer(l1);

        assert_eq!(doc.layers.len(), 1);
        let dup_id = doc.duplicate_layer(l1_id).expect("Duplicate failed");
        assert_eq!(doc.layers.len(), 2);
        assert_eq!(doc.layers[1].name, "Camada 1 (Cópia)");
        assert_eq!(doc.active_layer_id, Some(dup_id));

        doc.flip_layer_horizontal(l1_id);
        assert!(doc.find_layer(l1_id).unwrap().transform.flip_x);

        doc.flip_canvas_horizontal();
        let g = doc.add_guide(GuideAxis::Vertical, 200.0);
        assert_eq!(doc.guides.len(), 1);
        assert_eq!(g.position, 200.0);

        doc.resize_canvas(1000, 800, 100.0, 100.0);
        assert_eq!(doc.width, 1000);
        assert_eq!(doc.height, 800);
        assert_eq!(doc.guides[0].position, 300.0);

        doc.clear_guides();
        assert_eq!(doc.guides.len(), 0);
    }

    #[test]
    fn rejects_invalid_parent_and_clipping_graphs_without_mutating_document() {
        let mut doc = Document::new(10, 10, 72.0);
        let base = Layer::new_pixel(
            "base".to_string(),
            1,
            1,
            LayerTransform::new(Point::ZERO, Size::new(1.0, 1.0)),
        );
        let base_id = base.id;
        doc.try_add_layer(base).unwrap();

        let mut child = Layer::new_pixel(
            "child".to_string(),
            1,
            1,
            LayerTransform::new(Point::ZERO, Size::new(1.0, 1.0)),
        );
        child.parent_id = Some(base_id);
        assert_eq!(
            doc.try_add_layer(child),
            Err(LayerGraphError::ParentIsNotGroup)
        );
        assert_eq!(doc.layers.len(), 1);

        let mut clipped = Layer::new_pixel(
            "clipped".to_string(),
            1,
            1,
            LayerTransform::new(Point::ZERO, Size::new(1.0, 1.0)),
        );
        clipped.clipping_base_id = Some(Uuid::new_v4());
        assert_eq!(
            doc.try_add_layer(clipped),
            Err(LayerGraphError::MissingLayer)
        );
        assert_eq!(doc.layers.len(), 1);
    }

    #[test]
    fn removing_group_repairs_children_and_clipping_references() {
        let mut doc = Document::new(10, 10, 72.0);
        let group = Layer::new_group(
            "group".to_string(),
            LayerTransform::new(Point::ZERO, Size::new(1.0, 1.0)),
        );
        let group_id = group.id;
        doc.try_add_layer(group).unwrap();

        let mut base = Layer::new_pixel(
            "base".to_string(),
            1,
            1,
            LayerTransform::new(Point::ZERO, Size::new(1.0, 1.0)),
        );
        base.parent_id = Some(group_id);
        let base_id = base.id;
        doc.try_add_layer(base).unwrap();

        let mut clipped = Layer::new_pixel(
            "clipped".to_string(),
            1,
            1,
            LayerTransform::new(Point::ZERO, Size::new(1.0, 1.0)),
        );
        clipped.clipping_base_id = Some(base_id);
        doc.try_add_layer(clipped).unwrap();

        doc.remove_layer(group_id);
        assert_eq!(doc.find_layer(base_id).unwrap().parent_id, None);
        doc.remove_layer(base_id);
        assert_eq!(doc.active_layer().unwrap().clipping_base_id, None);
        assert_eq!(doc.validate_layer_graph(), Ok(()));
    }
}
