//! Controle de Câmera, Projeção e Viewport do Canvas (Zoom, Pan, Rotação).
//! Traduzido de Compositor/Rendering/CanvasViewport.swift.

use crate::core::transform::Point;

#[derive(Debug, Clone, PartialEq)]
pub struct CanvasViewport {
    pub pan_x: f64,
    pub pan_y: f64,
    pub zoom: f64,
    pub rotation: f64,
    pub view_width: f64,
    pub view_height: f64,
}

impl Default for CanvasViewport {
    fn default() -> Self {
        Self {
            pan_x: 0.0,
            pan_y: 0.0,
            zoom: 1.0,
            rotation: 0.0,
            view_width: 800.0,
            view_height: 600.0,
        }
    }
}

impl CanvasViewport {
    pub fn new(view_width: f64, view_height: f64) -> Self {
        Self {
            view_width,
            view_height,
            ..Default::default()
        }
    }

    /// Centraliza o documento de dimensões `doc_width` x `doc_height` no viewport.
    pub fn fit_document(&mut self, doc_width: f64, doc_height: f64) {
        if doc_width <= 0.0 || doc_height <= 0.0 {
            return;
        }
        let padding = 40.0;
        let available_w = (self.view_width - padding * 2.0).max(10.0);
        let available_h = (self.view_height - padding * 2.0).max(10.0);

        let scale_w = available_w / doc_width;
        let scale_h = available_h / doc_height;
        self.zoom = scale_w.min(scale_h).clamp(0.02, 64.0);

        self.pan_x = (self.view_width - doc_width * self.zoom) / 2.0;
        self.pan_y = (self.view_height - doc_height * self.zoom) / 2.0;
        self.rotation = 0.0;
    }

    /// Aplica deslocamento (Pan) em pixels de tela.
    pub fn pan(&mut self, dx: f64, dy: f64) {
        self.pan_x += dx;
        self.pan_y += dy;
    }

    /// Aplica Zoom centrado nas coordenadas de tela do cursor (screen_x, screen_y).
    pub fn zoom_at(&mut self, screen_x: f64, screen_y: f64, factor: f64) {
        let old_zoom = self.zoom;
        let new_zoom = (old_zoom * factor).clamp(0.01, 64.0);
        let actual_factor = new_zoom / old_zoom;

        self.pan_x = screen_x - (screen_x - self.pan_x) * actual_factor;
        self.pan_y = screen_y - (screen_y - self.pan_y) * actual_factor;
        self.zoom = new_zoom;
    }

    /// Converte coordenadas de tela (pixels da janela GTK) para coordenadas do documento.
    pub fn screen_to_document(&self, screen_x: f64, screen_y: f64) -> Point {
        Point {
            x: (screen_x - self.pan_x) / self.zoom,
            y: (screen_y - self.pan_y) / self.zoom,
        }
    }

    /// Converte coordenadas do documento para coordenadas de tela.
    pub fn document_to_screen(&self, doc_x: f64, doc_y: f64) -> Point {
        Point {
            x: doc_x * self.zoom + self.pan_x,
            y: doc_y * self.zoom + self.pan_y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_conversion() {
        let mut vp = CanvasViewport::new(1000.0, 1000.0);
        vp.pan_x = 100.0;
        vp.pan_y = 100.0;
        vp.zoom = 2.0;

        let screen_pt = vp.document_to_screen(50.0, 50.0);
        assert_eq!(screen_pt.x, 200.0);
        assert_eq!(screen_pt.y, 200.0);

        let doc_pt = vp.screen_to_document(200.0, 200.0);
        assert_eq!(doc_pt.x, 50.0);
        assert_eq!(doc_pt.y, 50.0);
    }
}
