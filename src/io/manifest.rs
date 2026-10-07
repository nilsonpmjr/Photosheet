//! Estrutura e Serialização do manifest.json do formato .comp (versões 1 a 11).
//! Traduzido de Compositor/IO/ProjectStore.swift e docs/project-format.md.

use crate::core::adjustment::LayerAdjustment;
use crate::core::blend::LayerBlendMode;
use crate::core::document::CanvasGuide;
use crate::core::effects::LayerEffects;
use crate::core::shape::LayerShapeStyle;
use crate::core::text::LayerTextStyle;
use crate::core::transform::LayerTransform;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const CURRENT_FORMAT_VERSION: u32 = 11;
pub const SUPPORTED_VERSIONS_MIN: u32 = 1;
pub const SUPPORTED_VERSIONS_MAX: u32 = 11;

fn default_format() -> String {
    "com.compositor.project".to_string()
}

fn default_colorspace() -> String {
    "sRGB".to_string()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectManifest {
    #[serde(default = "default_format")]
    pub format: String,
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default = "default_colorspace", rename = "colorSpace")]
    pub color_space: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<f64>,
    #[serde(rename = "documentID")]
    pub document_id: Uuid,
    pub width: usize,
    pub height: usize,
    #[serde(rename = "activeLayerID")]
    pub active_layer_id: Option<Uuid>,
    pub layers: Vec<ProjectLayerRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guides: Option<Vec<CanvasGuide>>,
}

fn default_version() -> u32 {
    CURRENT_FORMAT_VERSION
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectLayerRecord {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "isVisible")]
    pub is_visible: bool,
    pub transform: LayerTransform,
    #[serde(rename = "imageFile", skip_serializing_if = "Option::is_none")]
    pub image_file: Option<String>,
    #[serde(rename = "parentID", skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<Uuid>,
    #[serde(rename = "isGroup", skip_serializing_if = "Option::is_none")]
    pub is_group: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f64>,
    #[serde(rename = "blendMode", skip_serializing_if = "Option::is_none")]
    pub blend_mode: Option<LayerBlendMode>,
    #[serde(rename = "maskFile", skip_serializing_if = "Option::is_none")]
    pub mask_file: Option<String>,
    #[serde(rename = "maskEnabled", skip_serializing_if = "Option::is_none")]
    pub mask_enabled: Option<bool>,
    #[serde(rename = "maskSourceID", skip_serializing_if = "Option::is_none")]
    pub mask_source_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjustment: Option<LayerAdjustment>,
    #[serde(rename = "maskPlacement", skip_serializing_if = "Option::is_none")]
    pub mask_placement: Option<LayerTransform>,
    #[serde(rename = "maskLinked", skip_serializing_if = "Option::is_none")]
    pub mask_linked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape: Option<LayerShapeStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effects: Option<LayerEffects>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<LayerTextStyle>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::transform::{Point, Size};

    #[test]
    fn test_manifest_roundtrip() {
        let doc_id = Uuid::new_v4();
        let layer_id = Uuid::new_v4();

        let manifest = ProjectManifest {
            format: "com.compositor.project".to_string(),
            version: 11,
            color_space: "sRGB".to_string(),
            resolution: Some(300.0),
            document_id: doc_id,
            width: 1920,
            height: 1080,
            active_layer_id: Some(layer_id),
            layers: vec![ProjectLayerRecord {
                id: layer_id,
                name: "Background".to_string(),
                is_visible: true,
                transform: LayerTransform::new(Point::ZERO, Size::new(1920.0, 1080.0)),
                image_file: Some(format!("{}.png", layer_id)),
                parent_id: None,
                is_group: Some(false),
                opacity: Some(1.0),
                blend_mode: Some(LayerBlendMode::Normal),
                mask_file: None,
                mask_enabled: None,
                mask_source_id: None,
                adjustment: None,
                mask_placement: None,
                mask_linked: None,
                shape: None,
                effects: None,
                text: None,
            }],
            guides: None,
        };

        let json = serde_json::to_string_pretty(&manifest).unwrap();
        let decoded: ProjectManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(manifest, decoded);
    }
}
