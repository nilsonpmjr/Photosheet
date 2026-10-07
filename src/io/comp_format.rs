//! Leitura e Gravação Atômica de Pacotes .comp.
//! Traduzido de Compositor/IO/ProjectStore.swift.

use crate::core::document::Document;
use crate::core::layer::{Layer, LayerKind};
use crate::core::mask::LayerMask;
use crate::io::manifest::{ProjectLayerRecord, ProjectManifest, CURRENT_FORMAT_VERSION, SUPPORTED_VERSIONS_MAX, SUPPORTED_VERSIONS_MIN};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug)]
pub enum CompError {
    Io(std::io::Error),
    Json(serde_json::Error),
    UnsupportedVersion(u32),
    InvalidFormat(String),
    MissingImage(String),
}

impl From<std::io::Error> for CompError {
    fn from(e: std::io::Error) -> Self {
        CompError::Io(e)
    }
}

impl From<serde_json::Error> for CompError {
    fn from(e: serde_json::Error) -> Self {
        CompError::Json(e)
    }
}

pub struct CompPackage;

impl CompPackage {
    /// Carrega um pacote .comp do disco, decodificando o manifesto e os assets em memória.
    pub fn load(package_path: &Path) -> Result<(Document, HashMap<Uuid, Vec<u8>>), CompError> {
        let manifest_path = package_path.join("manifest.json");
        let manifest_str = fs::read_to_string(&manifest_path)?;
        let manifest: ProjectManifest = serde_json::from_str(&manifest_str)?;

        if manifest.version < SUPPORTED_VERSIONS_MIN || manifest.version > SUPPORTED_VERSIONS_MAX {
            return Err(CompError::UnsupportedVersion(manifest.version));
        }

        let mut doc = Document::new(manifest.width, manifest.height, manifest.resolution.unwrap_or(72.0));
        doc.id = manifest.document_id;
        doc.active_layer_id = manifest.active_layer_id;
        if let Some(guides) = manifest.guides {
            doc.guides = guides;
        }

        let images_dir = package_path.join("images");
        let mut image_assets = HashMap::new();

        for record in manifest.layers {
            let mut layer_mask = None;
            if let Some(ref mask_file) = record.mask_file {
                let mask_path = images_dir.join(mask_file);
                if mask_path.exists() {
                    let mask_bytes = fs::read(&mask_path)?;
                    // Se houver imagem PNG válida, pode ser decodificada
                    layer_mask = Some(LayerMask {
                        data: mask_bytes,
                        width: record.transform.size.width as usize,
                        height: record.transform.size.height as usize,
                        enabled: record.mask_enabled.unwrap_or(true),
                        linked: record.mask_linked.unwrap_or(true),
                        placement: record.mask_placement,
                    });
                }
            }

            let kind = if record.is_group == Some(true) {
                LayerKind::Group
            } else if let Some(adj) = record.adjustment {
                LayerKind::Adjustment(adj)
            } else if let Some(shape) = record.shape {
                LayerKind::Shape(shape)
            } else if let Some(text) = record.text {
                LayerKind::Text(text)
            } else {
                let mut pixels = None;
                if let Some(ref img_file) = record.image_file {
                    let img_path = images_dir.join(img_file);
                    if img_path.exists() {
                        let bytes = fs::read(&img_path)?;
                        image_assets.insert(record.id, bytes.clone());
                        pixels = Some(bytes);
                    }
                }
                LayerKind::Pixel {
                    image_file: record.image_file.clone(),
                    pixels,
                    width: record.transform.size.width as usize,
                    height: record.transform.size.height as usize,
                }
            };

            let layer = Layer {
                id: record.id,
                name: record.name,
                is_visible: record.is_visible,
                is_locked: false,
                opacity: record.opacity.unwrap_or(1.0),
                blend_mode: record.blend_mode.unwrap_or_default(),
                transform: record.transform,
                parent_id: record.parent_id,
                is_group: record.is_group.unwrap_or(false),
                mask: layer_mask,
                clipping_base_id: record.mask_source_id,
                effects: record.effects,
                kind,
            };

            doc.layers.push(layer);
        }

        Ok((doc, image_assets))
    }

    /// Salva um pacote .comp de forma atômica no disco.
    pub fn save(
        doc: &Document,
        image_assets: &HashMap<Uuid, Vec<u8>>,
        target_path: &Path,
    ) -> Result<(), CompError> {
        let tmp_path = target_path.with_extension(format!("tmp-{}", Uuid::new_v4()));
        fs::create_dir_all(&tmp_path)?;
        let images_dir = tmp_path.join("images");
        fs::create_dir_all(&images_dir)?;

        let mut manifest_layers = Vec::new();

        for layer in &doc.layers {
            let mut image_file = None;
            let mut adjustment = None;
            let mut shape = None;
            let mut text = None;

            match &layer.kind {
                LayerKind::Pixel { image_file: orig_file, .. } => {
                    let filename = orig_file.clone().unwrap_or_else(|| format!("{}.png", layer.id));
                    if let Some(bytes) = image_assets.get(&layer.id) {
                        fs::write(images_dir.join(&filename), bytes)?;
                        image_file = Some(filename);
                    }
                }
                LayerKind::Adjustment(adj) => {
                    adjustment = Some(adj.clone());
                }
                LayerKind::Shape(s) => {
                    shape = Some(s.clone());
                }
                LayerKind::Text(t) => {
                    text = Some(t.clone());
                }
                LayerKind::Group => {}
            }

            let mut mask_file = None;
            let mut mask_enabled = None;
            let mut mask_linked = None;
            let mut mask_placement = None;

            if let Some(ref m) = layer.mask {
                let m_name = format!("{}.mask.png", layer.id);
                fs::write(images_dir.join(&m_name), &m.data)?;
                mask_file = Some(m_name);
                mask_enabled = Some(m.enabled);
                mask_linked = Some(m.linked);
                mask_placement = m.placement.clone();
            }

            manifest_layers.push(ProjectLayerRecord {
                id: layer.id,
                name: layer.name.clone(),
                is_visible: layer.is_visible,
                transform: layer.transform.clone(),
                image_file,
                parent_id: layer.parent_id,
                is_group: if layer.is_group { Some(true) } else { None },
                opacity: Some(layer.opacity),
                blend_mode: Some(layer.blend_mode),
                mask_file,
                mask_enabled,
                mask_source_id: layer.clipping_base_id,
                adjustment,
                mask_placement,
                mask_linked,
                shape,
                effects: layer.effects.clone(),
                text,
            });
        }

        let manifest = ProjectManifest {
            format: "com.compositor.project".to_string(),
            version: CURRENT_FORMAT_VERSION,
            color_space: "sRGB".to_string(),
            resolution: Some(doc.resolution),
            document_id: doc.id,
            width: doc.width,
            height: doc.height,
            active_layer_id: doc.active_layer_id,
            layers: manifest_layers,
            guides: if doc.guides.is_empty() { None } else { Some(doc.guides.clone()) },
        };

        let json_str = serde_json::to_string_pretty(&manifest)?;
        fs::write(tmp_path.join("manifest.json"), json_str)?;

        // Substituição atômica: se target_path já existia, remove antes de renomear
        if target_path.exists() {
            fs::remove_dir_all(target_path)?;
        }
        fs::rename(&tmp_path, target_path)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::transform::{LayerTransform, Point, Size};

    #[test]
    fn test_comp_package_save_and_load() {
        let temp_dir = std::env::temp_dir().join(format!("test-comp-{}", Uuid::new_v4()));
        let comp_path = temp_dir.join("test_project.comp");

        let mut doc = Document::new(800, 600, 72.0);
        let layer = Layer::new_pixel(
            "Layer 1".to_string(),
            800,
            600,
            LayerTransform::new(Point::ZERO, Size::new(800.0, 600.0)),
        );
        let layer_id = layer.id;
        doc.add_layer(layer);

        let mut assets = HashMap::new();
        assets.insert(layer_id, vec![255u8; 100]); // Mock PNG bytes

        CompPackage::save(&doc, &assets, &comp_path).expect("Failed to save .comp");
        assert!(comp_path.join("manifest.json").exists());

        let (loaded_doc, loaded_assets) = CompPackage::load(&comp_path).expect("Failed to load .comp");
        assert_eq!(loaded_doc.id, doc.id);
        assert_eq!(loaded_doc.width, 800);
        assert_eq!(loaded_doc.height, 600);
        assert_eq!(loaded_doc.layers.len(), 1);
        assert_eq!(loaded_doc.layers[0].name, "Layer 1");
        assert!(loaded_assets.contains_key(&layer_id));

        let _ = fs::remove_dir_all(temp_dir);
    }
}
