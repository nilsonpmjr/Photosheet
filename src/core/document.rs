//! Modelo de Documento em Memória.
//! Traduzido de Compositor/Document/ProjectWorkspace.swift e Document.

use crate::core::layer::Layer;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

    pub fn add_layer(&mut self, layer: Layer) {
        let id = layer.id;
        self.layers.push(layer);
        self.active_layer_id = Some(id);
    }

    pub fn remove_layer(&mut self, id: Uuid) -> Option<Layer> {
        if let Some(pos) = self.layers.iter().position(|l| l.id == id) {
            let removed = self.layers.remove(pos);
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
}
