//! Sessão de Edição (EditorSession).
//! Traduzido de Compositor/Document/EditorSession.swift, EditorSession+Brush.swift e ToolDefaults.swift.

use crate::core::adjustment::LayerAdjustment;
use crate::core::blend::LayerBlendMode;
use crate::core::document::Document;
use crate::core::history::DocumentHistory;
use crate::core::layer::Layer;
use crate::core::transform::{LayerTransform, Point, Size};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationTool {
    Move,
    Marquee,
    Lasso,
    Wand,
    Crop,
    Brush,
    Eraser,
    SpotHealing,
    CloneStamp,
    Blur,
    Gradient,
    Shape,
    Type,
    Eyedropper,
    Hand,
    Zoom,
    Idle,
}

impl NavigationTool {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Move => "Mover (V)",
            Self::Marquee => "Seleção Retangular (M)",
            Self::Lasso => "Laço (L)",
            Self::Wand => "Varinha Mágica (W)",
            Self::Crop => "Corte Demarcatório (C)",
            Self::Brush => "Pincel (B)",
            Self::Eraser => "Borracha (E)",
            Self::SpotHealing => "Pincel de Recuperação (J)",
            Self::CloneStamp => "Carimbo (S)",
            Self::Blur => "Desfoque / Smear (R)",
            Self::Gradient => "Gradiente (G)",
            Self::Shape => "Forma Vetorial (U)",
            Self::Type => "Texto (T)",
            Self::Eyedropper => "Conta-gotas (I)",
            Self::Hand => "Mão (H)",
            Self::Zoom => "Zoom (Z)",
            Self::Idle => "Inativo (A)",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::Move => "transform-move-symbolic",
            Self::Marquee => "select-rectangular-symbolic",
            Self::Lasso => "select-lasso-symbolic",
            Self::Wand => "select-magic-wand-symbolic",
            Self::Crop => "tool-crop-symbolic",
            Self::Brush => "draw-brush-symbolic",
            Self::Eraser => "draw-eraser-symbolic",
            Self::SpotHealing => "bandage-symbolic",
            Self::CloneStamp => "clone-stamp-symbolic",
            Self::Blur => "blur-symbolic",
            Self::Gradient => "color-gradient-symbolic",
            Self::Shape => "draw-rectangle-symbolic",
            Self::Type => "format-text-bold-symbolic",
            Self::Eyedropper => "color-picker-symbolic",
            Self::Hand => "open-menu-symbolic", // Fallback standard icon
            Self::Zoom => "zoom-in-symbolic",
            Self::Idle => "edit-clear-symbolic",
        }
    }

    pub fn shortcut_key(&self) -> char {
        match self {
            Self::Move => 'v',
            Self::Marquee => 'm',
            Self::Lasso => 'l',
            Self::Wand => 'w',
            Self::Crop => 'c',
            Self::Brush => 'b',
            Self::Eraser => 'e',
            Self::SpotHealing => 'j',
            Self::CloneStamp => 's',
            Self::Blur => 'r',
            Self::Gradient => 'g',
            Self::Shape => 'u',
            Self::Type => 't',
            Self::Eyedropper => 'i',
            Self::Hand => 'h',
            Self::Zoom => 'z',
            Self::Idle => 'a',
        }
    }
}

#[derive(Debug, Clone)]
pub struct BrushSettings {
    pub size: f64,
    pub hardness: f64,
    pub opacity: f64,
    pub flow: f64,
    pub spacing: f64,
    pub smoothing: f64,
}

impl Default for BrushSettings {
    fn default() -> Self {
        Self {
            size: 30.0,
            hardness: 0.8,
            opacity: 1.0,
            flow: 1.0,
            spacing: 0.25,
            smoothing: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WandSettings {
    pub tolerance: f64,
    pub contiguous: bool,
    pub antialias: bool,
}

impl Default for WandSettings {
    fn default() -> Self {
        Self {
            tolerance: 32.0,
            contiguous: true,
            antialias: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    Replace,
    Add,
    Subtract,
    Intersect,
}

#[derive(Debug, Clone)]
pub struct SelectionSettings {
    pub mode: SelectionMode,
    pub feather: f64,
    pub antialias: bool,
}

impl Default for SelectionSettings {
    fn default() -> Self {
        Self {
            mode: SelectionMode::Replace,
            feather: 0.0,
            antialias: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextToolSettings {
    pub font_family: String,
    pub font_size: f64,
    pub color: [u8; 4],
}

impl Default for TextToolSettings {
    fn default() -> Self {
        Self {
            font_family: "Sans".to_string(),
            font_size: 36.0,
            color: [0, 0, 0, 255],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeKind {
    Rectangle,
    Ellipse,
    Line,
}

#[derive(Debug, Clone)]
pub struct ShapeToolSettings {
    pub kind: ShapeKind,
    pub fill_color: [u8; 4],
    pub stroke_color: [u8; 4],
    pub stroke_width: f64,
    pub corner_radius: f64,
}

impl Default for ShapeToolSettings {
    fn default() -> Self {
        Self {
            kind: ShapeKind::Rectangle,
            fill_color: [0, 122, 255, 255],
            stroke_color: [0, 0, 0, 0],
            stroke_width: 0.0,
            corner_radius: 0.0,
        }
    }
}

pub struct EditorSession {
    pub document: Option<Document>,
    pub file_path: Option<PathBuf>,
    pub is_dirty: bool,
    pub active_tool: NavigationTool,
    pub foreground_color: [u8; 4],
    pub background_color: [u8; 4],
    pub brush_settings: BrushSettings,
    pub wand_settings: WandSettings,
    pub selection_settings: SelectionSettings,
    pub text_settings: TextToolSettings,
    pub shape_settings: ShapeToolSettings,
    pub history: DocumentHistory,
    pub zoom: f64,
    pub pan_x: f64,
    pub pan_y: f64,
}

impl EditorSession {
    pub fn new() -> Self {
        Self {
            document: None,
            file_path: None,
            is_dirty: false,
            active_tool: NavigationTool::Move,
            foreground_color: [0, 0, 0, 255],       // Preto
            background_color: [255, 255, 255, 255], // Branco
            brush_settings: BrushSettings::default(),
            wand_settings: WandSettings::default(),
            selection_settings: SelectionSettings::default(),
            text_settings: TextToolSettings::default(),
            shape_settings: ShapeToolSettings::default(),
            history: DocumentHistory::new(50),
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
        }
    }

    pub fn with_document(doc: Document) -> Self {
        let mut session = Self::new();
        session.set_document(doc);
        session
    }

    pub fn set_document(&mut self, doc: Document) {
        self.history.clear();
        self.document = Some(doc);
        self.is_dirty = false;
        self.zoom = 1.0;
        self.pan_x = 0.0;
        self.pan_y = 0.0;
    }

    pub fn swap_colors(&mut self) {
        std::mem::swap(&mut self.foreground_color, &mut self.background_color);
    }

    pub fn reset_colors(&mut self) {
        self.foreground_color = [0, 0, 0, 255];
        self.background_color = [255, 255, 255, 255];
    }

    pub fn push_history(&mut self, description: &str) {
        if let Some(doc) = &self.document {
            self.history.push(description.to_string(), doc);
            self.is_dirty = true;
        }
    }

    pub fn undo(&mut self) -> bool {
        let doc = match &self.document {
            Some(d) => d,
            None => return false,
        };
        if let Some(previous) = self.history.undo(doc) {
            self.document = Some(previous);
            self.is_dirty = true;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        let doc = match &self.document {
            Some(d) => d,
            None => return false,
        };
        if let Some(next) = self.history.redo(doc) {
            self.document = Some(next);
            self.is_dirty = true;
            true
        } else {
            false
        }
    }

    // --- Ações de Camadas ---

    pub fn add_empty_layer(&mut self, name: Option<String>) -> Option<Uuid> {
        let (width, height) = {
            let doc = self.document.as_ref()?;
            (doc.width, doc.height)
        };
        let layer_count = self.document.as_ref()?.layers.len();
        let layer_name = name.unwrap_or_else(|| format!("Camada {}", layer_count + 1));
        self.push_history("Nova Camada");
        let layer = Layer::new_pixel(
            layer_name,
            width,
            height,
            LayerTransform::new(
                Point { x: 0.0, y: 0.0 },
                Size {
                    width: width as f64,
                    height: height as f64,
                },
            ),
        );
        let id = layer.id;

        if let Some(doc) = self.document.as_mut() {
            doc.add_layer(layer);
        }
        Some(id)
    }

    pub fn add_group(&mut self, name: Option<String>) -> Option<Uuid> {
        let (width, height) = {
            let doc = self.document.as_ref()?;
            (doc.width, doc.height)
        };
        let layer_count = self.document.as_ref()?.layers.len();
        let group_name = name.unwrap_or_else(|| format!("Grupo {}", layer_count + 1));
        self.push_history("Novo Grupo");
        let group = Layer::new_group(
            group_name,
            LayerTransform::new(
                Point { x: 0.0, y: 0.0 },
                Size {
                    width: width as f64,
                    height: height as f64,
                },
            ),
        );
        let id = group.id;

        if let Some(doc) = self.document.as_mut() {
            doc.add_layer(group);
        }
        Some(id)
    }

    pub fn add_adjustment_layer(
        &mut self,
        adjustment: LayerAdjustment,
        name: Option<String>,
    ) -> Option<Uuid> {
        let (width, height) = {
            let doc = self.document.as_ref()?;
            (doc.width, doc.height)
        };
        let adj_name = name.unwrap_or_else(|| format!("Ajuste {:?}", adjustment));
        self.push_history("Nova Camada de Ajuste");
        let layer = Layer::new_adjustment(
            adj_name,
            adjustment,
            LayerTransform::new(
                Point { x: 0.0, y: 0.0 },
                Size {
                    width: width as f64,
                    height: height as f64,
                },
            ),
        );
        let id = layer.id;

        if let Some(doc) = self.document.as_mut() {
            doc.add_layer(layer);
        }
        Some(id)
    }

    pub fn delete_active_layer(&mut self) -> bool {
        let active_id = match self.document.as_ref().and_then(|d| d.active_layer_id) {
            Some(id) => id,
            None => return false,
        };

        if let Some(doc) = self.document.as_mut() {
            doc.remove_layer(active_id);
        }
        self.push_history("Excluir Camada");
        true
    }

    pub fn duplicate_active_layer(&mut self) -> Option<Uuid> {
        let doc = self.document.as_ref()?;
        let active = doc.active_layer()?.clone();
        let mut clone = active;
        clone.id = Uuid::new_v4();
        clone.name = format!("{} (cópia)", clone.name);
        let clone_id = clone.id;

        if let Some(doc) = self.document.as_mut() {
            doc.add_layer(clone);
        }
        self.push_history("Duplicar Camada");
        Some(clone_id)
    }

    pub fn toggle_active_layer_visibility(&mut self) -> bool {
        if let Some(doc) = self.document.as_mut() {
            if let Some(layer) = doc.active_layer_mut() {
                layer.is_visible = !layer.is_visible;
                let visible = layer.is_visible;
                self.is_dirty = true;
                return visible;
            }
        }
        false
    }

    pub fn set_active_layer_opacity(&mut self, opacity: f64) {
        if let Some(doc) = self.document.as_mut() {
            if let Some(layer) = doc.active_layer_mut() {
                layer.opacity = opacity.clamp(0.0, 1.0);
                self.is_dirty = true;
            }
        }
    }

    pub fn set_active_layer_blend_mode(&mut self, mode: LayerBlendMode) {
        if let Some(doc) = self.document.as_mut() {
            if let Some(layer) = doc.active_layer_mut() {
                layer.blend_mode = mode;
                self.is_dirty = true;
            }
        }
    }

    pub fn move_active_layer_up(&mut self) -> bool {
        let doc = match self.document.as_mut() {
            Some(d) => d,
            None => return false,
        };
        let active_id = match doc.active_layer_id {
            Some(id) => id,
            None => return false,
        };
        if let Some(pos) = doc.layers.iter().position(|l| l.id == active_id) {
            if pos + 1 < doc.layers.len() {
                doc.layers.swap(pos, pos + 1);
                self.push_history("Mover Camada para Cima");
                return true;
            }
        }
        false
    }

    pub fn move_active_layer_down(&mut self) -> bool {
        let doc = match self.document.as_mut() {
            Some(d) => d,
            None => return false,
        };
        let active_id = match doc.active_layer_id {
            Some(id) => id,
            None => return false,
        };
        if let Some(pos) = doc.layers.iter().position(|l| l.id == active_id) {
            if pos > 0 {
                doc.layers.swap(pos, pos - 1);
                self.push_history("Mover Camada para Baixo");
                return true;
            }
        }
        false
    }
}

impl Default for EditorSession {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_session_creation_and_layer_management() {
        let doc = Document::new(800, 600, 72.0);
        let mut session = EditorSession::with_document(doc);

        assert!(session.document.is_some());
        assert_eq!(session.document.as_ref().unwrap().layers.len(), 0);

        let l1 = session.add_empty_layer(Some("Fundo".into())).unwrap();
        assert_eq!(session.document.as_ref().unwrap().layers.len(), 1);
        assert_eq!(session.document.as_ref().unwrap().active_layer_id, Some(l1));

        let l2 = session.add_empty_layer(Some("Pintura".into())).unwrap();
        assert_eq!(session.document.as_ref().unwrap().layers.len(), 2);
        assert_eq!(session.document.as_ref().unwrap().active_layer_id, Some(l2));

        // Teste de opacidade e blend mode
        session.set_active_layer_opacity(0.5);
        assert_eq!(
            session
                .document
                .as_ref()
                .unwrap()
                .active_layer()
                .unwrap()
                .opacity,
            0.5
        );

        session.set_active_layer_blend_mode(LayerBlendMode::Multiply);
        assert_eq!(
            session
                .document
                .as_ref()
                .unwrap()
                .active_layer()
                .unwrap()
                .blend_mode,
            LayerBlendMode::Multiply
        );

        // Teste de Undo / Redo
        assert!(session.undo());
        assert_eq!(
            session
                .document
                .as_ref()
                .unwrap()
                .active_layer()
                .unwrap()
                .blend_mode,
            LayerBlendMode::Normal
        );

        assert!(session.redo());
        assert_eq!(
            session
                .document
                .as_ref()
                .unwrap()
                .active_layer()
                .unwrap()
                .blend_mode,
            LayerBlendMode::Multiply
        );

        // Teste de exclusão
        assert!(session.delete_active_layer());
        assert_eq!(session.document.as_ref().unwrap().layers.len(), 1);
    }

    #[test]
    fn test_color_swapping() {
        let mut session = EditorSession::new();
        assert_eq!(session.foreground_color, [0, 0, 0, 255]);
        assert_eq!(session.background_color, [255, 255, 255, 255]);

        session.swap_colors();
        assert_eq!(session.foreground_color, [255, 255, 255, 255]);
        assert_eq!(session.background_color, [0, 0, 0, 255]);

        session.reset_colors();
        assert_eq!(session.foreground_color, [0, 0, 0, 255]);
        assert_eq!(session.background_color, [255, 255, 255, 255]);
    }
}
