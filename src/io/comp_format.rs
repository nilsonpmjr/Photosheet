//! Leitura e Gravação Atômica de Pacotes .comp.
//! Traduzido de Compositor/IO/ProjectStore.swift.

use crate::core::document::Document;
use crate::core::layer::{Layer, LayerKind};
use crate::core::limits::DocumentLimits;
use crate::core::mask::LayerMask;
use crate::io::manifest::{
    ProjectLayerRecord, ProjectManifest, CURRENT_FORMAT_VERSION, SUPPORTED_VERSIONS_MAX,
    SUPPORTED_VERSIONS_MIN,
};
use image::{ImageBuffer, ImageFormat, ImageReader, Luma, Rgba};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug)]
pub enum CompError {
    Io(std::io::Error),
    Json(serde_json::Error),
    UnsupportedVersion(u32),
    InvalidFormat(String),
    MissingImage(String),
    InvalidImage(String),
    InvalidManifest(String),
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
    pub fn load(package_path: &Path) -> Result<Document, CompError> {
        let manifest_path = package_path.join("manifest.json");
        let manifest_str = fs::read_to_string(&manifest_path)?;
        let manifest: ProjectManifest = serde_json::from_str(&manifest_str)?;

        if manifest.version < SUPPORTED_VERSIONS_MIN || manifest.version > SUPPORTED_VERSIONS_MAX {
            return Err(CompError::UnsupportedVersion(manifest.version));
        }
        validate_manifest(&manifest)?;

        let mut doc = Document::new(
            manifest.width,
            manifest.height,
            manifest.resolution.unwrap_or(72.0),
        );
        doc.id = manifest.document_id;
        doc.active_layer_id = manifest.active_layer_id;
        if let Some(guides) = manifest.guides {
            doc.guides = guides;
        }

        let images_dir = package_path.join("images");
        for record in manifest.layers {
            let mut layer_mask = None;
            if let Some(ref mask_file) = record.mask_file {
                let mask_path = images_dir.join(mask_file);
                let (data, width, height) = decode_mask_png(&mask_path)?;
                layer_mask = Some(LayerMask {
                    data,
                    width,
                    height,
                    enabled: record.mask_enabled.unwrap_or(true),
                    linked: record.mask_linked.unwrap_or(true),
                    placement: record.mask_placement,
                });
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
                let mut width = record.transform.size.width as usize;
                let mut height = record.transform.size.height as usize;
                if let Some(ref img_file) = record.image_file {
                    let img_path = images_dir.join(img_file);
                    let (data, image_width, image_height) = decode_rgba_png(&img_path)?;
                    pixels = Some(data);
                    width = image_width;
                    height = image_height;
                }
                LayerKind::Pixel {
                    image_file: record.image_file.clone(),
                    pixels,
                    width,
                    height,
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

        Ok(doc)
    }

    /// Salva um pacote .comp de forma atômica no disco.
    pub fn save(doc: &Document, target_path: &Path) -> Result<(), CompError> {
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
                LayerKind::Pixel {
                    pixels,
                    width,
                    height,
                    ..
                } => {
                    if let Some(pixels) = pixels {
                        let filename = image_filename(layer.id);
                        encode_rgba_png(pixels, *width, *height, &images_dir.join(&filename))?;
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
                let m_name = mask_filename(layer.id);
                encode_mask_png(&m.data, m.width, m.height, &images_dir.join(&m_name))?;
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
            guides: if doc.guides.is_empty() {
                None
            } else {
                Some(doc.guides.clone())
            },
        };
        validate_manifest(&manifest)?;

        let json_str = serde_json::to_string_pretty(&manifest)?;
        fs::write(tmp_path.join("manifest.json"), json_str)?;

        replace_package(&tmp_path, target_path)?;

        Ok(())
    }
}

fn image_filename(id: Uuid) -> String {
    format!("{}.png", id.to_string().to_uppercase())
}

fn mask_filename(id: Uuid) -> String {
    format!("{}.mask.png", id.to_string().to_uppercase())
}

fn decode_rgba_png(path: &Path) -> Result<(Vec<u8>, usize, usize), CompError> {
    let image = open_png(path)?;
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    Ok((rgba.into_raw(), width as usize, height as usize))
}

fn decode_mask_png(path: &Path) -> Result<(Vec<u8>, usize, usize), CompError> {
    let image = open_png(path)?;
    let gray = image.to_luma8();
    let (width, height) = gray.dimensions();
    Ok((gray.into_raw(), width as usize, height as usize))
}

fn open_png(path: &Path) -> Result<image::DynamicImage, CompError> {
    let reader = ImageReader::open(path)
        .map_err(|_| CompError::MissingImage(path.display().to_string()))?
        .with_guessed_format()
        .map_err(|_| CompError::InvalidImage(path.display().to_string()))?;
    if reader.format() != Some(ImageFormat::Png) {
        return Err(CompError::InvalidImage(format!(
            "{} não é um PNG",
            path.display()
        )));
    }
    reader
        .decode()
        .map_err(|_| CompError::InvalidImage(path.display().to_string()))
}

fn encode_rgba_png(
    pixels: &[u8],
    width: usize,
    height: usize,
    path: &Path,
) -> Result<(), CompError> {
    let expected = width
        .checked_mul(height)
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(|| {
            CompError::InvalidImage("Dimensões de imagem excedem o limite".to_string())
        })?;
    if pixels.len() != expected {
        return Err(CompError::InvalidImage(format!(
            "Buffer RGBA tem {} bytes; esperados {}",
            pixels.len(),
            expected
        )));
    }
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(width as u32, height as u32, pixels.to_vec())
        .ok_or_else(|| CompError::InvalidImage("Não foi possível criar imagem RGBA".to_string()))?;
    image
        .save_with_format(path, ImageFormat::Png)
        .map_err(|error| CompError::InvalidImage(error.to_string()))
}

fn encode_mask_png(data: &[u8], width: usize, height: usize, path: &Path) -> Result<(), CompError> {
    let expected = width.checked_mul(height).ok_or_else(|| {
        CompError::InvalidImage("Dimensões de máscara excedem o limite".to_string())
    })?;
    if data.len() != expected {
        return Err(CompError::InvalidImage(format!(
            "Buffer de máscara tem {} bytes; esperados {}",
            data.len(),
            expected
        )));
    }
    let image = ImageBuffer::<Luma<u8>, _>::from_raw(width as u32, height as u32, data.to_vec())
        .ok_or_else(|| CompError::InvalidImage("Não foi possível criar máscara".to_string()))?;
    image
        .save_with_format(path, ImageFormat::Png)
        .map_err(|error| CompError::InvalidImage(error.to_string()))
}

fn replace_package(staged_path: &Path, target_path: &Path) -> Result<(), CompError> {
    if !target_path.exists() {
        return fs::rename(staged_path, target_path).map_err(CompError::Io);
    }

    let backup_path = sibling_path(target_path, "backup");
    fs::rename(target_path, &backup_path)?;
    match fs::rename(staged_path, target_path) {
        Ok(()) => fs::remove_dir_all(backup_path).map_err(CompError::Io),
        Err(error) => {
            let _ = fs::rename(&backup_path, target_path);
            Err(CompError::Io(error))
        }
    }
}

fn sibling_path(path: &Path, suffix: &str) -> PathBuf {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("project");
    path.with_file_name(format!("{}.{}.{}", name, suffix, Uuid::new_v4()))
}

fn validate_manifest(manifest: &ProjectManifest) -> Result<(), CompError> {
    if manifest.format != "com.compositor.project" {
        return Err(CompError::InvalidFormat(manifest.format.clone()));
    }
    if manifest.color_space != "sRGB"
        || !(1..=DocumentLimits::MAX_SIDE).contains(&manifest.width)
        || !(1..=DocumentLimits::MAX_SIDE).contains(&manifest.height)
        || manifest.layers.len() > 10_000
    {
        return Err(CompError::InvalidManifest(
            "Canvas ou espaço de cor inválido".to_string(),
        ));
    }
    if let Some(resolution) = manifest.resolution {
        if !resolution.is_finite() || !(1.0..=9600.0).contains(&resolution) {
            return Err(CompError::InvalidManifest("Resolução inválida".to_string()));
        }
    }

    let mut records = std::collections::HashMap::with_capacity(manifest.layers.len());
    for record in &manifest.layers {
        if records.insert(record.id, record).is_some()
            || record.name.trim().is_empty()
            || record.name.len() > 16_384
            || !record.transform.is_valid()
        {
            return Err(CompError::InvalidManifest(
                "Camada inválida ou ID duplicado".to_string(),
            ));
        }
        if record
            .image_file
            .as_deref()
            .is_some_and(|name| name != image_filename(record.id))
        {
            return Err(CompError::InvalidManifest(
                "Nome de arquivo de imagem inválido".to_string(),
            ));
        }
        if record.is_group == Some(true) && record.image_file.is_some() {
            return Err(CompError::InvalidManifest(
                "Grupos não podem ter pixels próprios".to_string(),
            ));
        }
        validate_layer_version_fields(manifest.version, record)?;
    }

    if manifest
        .active_layer_id
        .is_some_and(|id| !records.contains_key(&id))
    {
        return Err(CompError::InvalidManifest(
            "Camada ativa ausente".to_string(),
        ));
    }
    validate_hierarchy(&manifest.layers, &records)?;
    validate_clipping_graph(&manifest.layers, &records)?;
    validate_guides(manifest)?;
    Ok(())
}

fn validate_layer_version_fields(
    version: u32,
    record: &ProjectLayerRecord,
) -> Result<(), CompError> {
    let is_group = record.is_group == Some(true);
    let opacity = record.opacity.unwrap_or(1.0);
    if !opacity.is_finite()
        || !(0.0..=1.0).contains(&opacity)
        || (version < 3
            && (opacity != 1.0
                || record
                    .blend_mode
                    .is_some_and(|mode| mode != Default::default())))
        || (is_group
            && (record
                .blend_mode
                .is_some_and(|mode| mode != Default::default())
                || (version < 8 && opacity != 1.0)))
    {
        return Err(CompError::InvalidManifest(
            "Opacidade ou blend mode inválido".to_string(),
        ));
    }

    if let Some(mask_file) = &record.mask_file {
        if version < if is_group { 6 } else { 4 } || mask_file != &mask_filename(record.id) {
            return Err(CompError::InvalidManifest("Máscara inválida".to_string()));
        }
    }
    if record.mask_enabled.is_some() && record.mask_file.is_none()
        || record.mask_placement.is_some()
            && (record.mask_file.is_none()
                || !record.mask_placement.as_ref().is_some_and(|p| p.is_valid()))
        || version < 5 && record.mask_source_id.is_some()
        || version == 1 && (record.parent_id.is_some() || is_group)
    {
        return Err(CompError::InvalidManifest(
            "Campo de máscara ou hierarquia não suportado nesta versão".to_string(),
        ));
    }

    if let Some(text) = &record.text {
        if version < 10 && text.color_runs.is_some() || version < 11 && text.font_runs.is_some() {
            return Err(CompError::InvalidManifest(
                "Runs de texto não suportados nesta versão".to_string(),
            ));
        }
    }
    Ok(())
}

fn validate_hierarchy(
    layers: &[ProjectLayerRecord],
    records: &std::collections::HashMap<Uuid, &ProjectLayerRecord>,
) -> Result<(), CompError> {
    for layer in layers {
        let mut seen = std::collections::HashSet::from([layer.id]);
        let mut parent = layer.parent_id;
        while let Some(id) = parent {
            let Some(node) = records.get(&id) else {
                return Err(CompError::InvalidManifest("Grupo pai ausente".to_string()));
            };
            if seen.len() > 64 || !seen.insert(id) || node.is_group != Some(true) {
                return Err(CompError::InvalidManifest(
                    "Ciclo ou profundidade inválida de grupos".to_string(),
                ));
            }
            parent = node.parent_id;
        }
    }
    Ok(())
}

fn validate_clipping_graph(
    layers: &[ProjectLayerRecord],
    records: &std::collections::HashMap<Uuid, &ProjectLayerRecord>,
) -> Result<(), CompError> {
    for layer in layers {
        let mut path = std::collections::HashSet::new();
        let mut current = Some(layer.id);
        while let Some(id) = current {
            if path.len() >= 256 || !path.insert(id) {
                return Err(CompError::InvalidManifest(
                    "Ciclo de máscara de recorte".to_string(),
                ));
            }
            let Some(record) = records.get(&id) else {
                return Err(CompError::InvalidManifest("Camada ausente".to_string()));
            };
            if let Some(source) = record.mask_source_id {
                let Some(source_record) = records.get(&source) else {
                    return Err(CompError::InvalidManifest(
                        "Fonte de máscara de recorte ausente".to_string(),
                    ));
                };
                if record.is_group == Some(true)
                    || source_record.is_group == Some(true)
                    || source_record.adjustment.is_some()
                {
                    return Err(CompError::InvalidManifest(
                        "Máscara de recorte não pode referenciar grupo ou ajuste".to_string(),
                    ));
                }
            }
            current = record.mask_source_id;
        }
    }
    Ok(())
}

fn validate_guides(manifest: &ProjectManifest) -> Result<(), CompError> {
    let guides = manifest.guides.as_deref().unwrap_or_default();
    if manifest.version < 8 && !guides.is_empty() || guides.len() > 1_000 {
        return Err(CompError::InvalidManifest("Guias inválidas".to_string()));
    }
    let mut ids = std::collections::HashSet::new();
    if guides.iter().any(|guide| {
        !ids.insert(guide.id) || !guide.position.is_finite() || guide.position.abs() > 1_000_000.0
    }) {
        return Err(CompError::InvalidManifest("Guia inválida".to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::blend::LayerBlendMode;
    use crate::core::transform::{LayerTransform, Point, Size};

    fn record(id: Uuid) -> ProjectLayerRecord {
        ProjectLayerRecord {
            id,
            name: "Layer".to_string(),
            is_visible: true,
            transform: LayerTransform::new(Point::ZERO, Size::new(1.0, 1.0)),
            image_file: Some(image_filename(id)),
            parent_id: None,
            is_group: None,
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
        }
    }

    fn manifest(layers: Vec<ProjectLayerRecord>) -> ProjectManifest {
        ProjectManifest {
            format: "com.compositor.project".to_string(),
            version: CURRENT_FORMAT_VERSION,
            color_space: "sRGB".to_string(),
            resolution: Some(72.0),
            document_id: Uuid::new_v4(),
            width: 100,
            height: 100,
            active_layer_id: None,
            layers,
            guides: None,
        }
    }

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

        CompPackage::save(&doc, &comp_path).expect("Failed to save .comp");
        assert!(comp_path.join("manifest.json").exists());
        assert!(comp_path
            .join("images")
            .join(image_filename(layer_id))
            .exists());

        let loaded_doc = CompPackage::load(&comp_path).expect("Failed to load .comp");
        assert_eq!(loaded_doc.id, doc.id);
        assert_eq!(loaded_doc.width, 800);
        assert_eq!(loaded_doc.height, 600);
        assert_eq!(loaded_doc.layers.len(), 1);
        assert_eq!(loaded_doc.layers[0].name, "Layer 1");
        let LayerKind::Pixel {
            pixels: Some(pixels),
            width,
            height,
            ..
        } = &loaded_doc.layers[0].kind
        else {
            panic!("Expected decoded pixel layer");
        };
        assert_eq!((*width, *height), (800, 600));
        assert_eq!(pixels.len(), 800 * 600 * 4);

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn rejects_invalid_group_and_clipping_graphs() {
        let child_id = Uuid::new_v4();
        let mut orphan = record(child_id);
        orphan.parent_id = Some(Uuid::new_v4());
        assert!(matches!(
            validate_manifest(&manifest(vec![orphan])),
            Err(CompError::InvalidManifest(_))
        ));

        let group_id = Uuid::new_v4();
        let mut group = record(group_id);
        group.is_group = Some(true);
        group.image_file = None;
        group.parent_id = Some(child_id);
        let mut child = record(child_id);
        child.parent_id = Some(group_id);
        assert!(matches!(
            validate_manifest(&manifest(vec![group, child])),
            Err(CompError::InvalidManifest(_))
        ));

        let first_id = Uuid::new_v4();
        let second_id = Uuid::new_v4();
        let mut first = record(first_id);
        first.mask_source_id = Some(second_id);
        let mut second = record(second_id);
        second.mask_source_id = Some(first_id);
        assert!(matches!(
            validate_manifest(&manifest(vec![first, second])),
            Err(CompError::InvalidManifest(_))
        ));
    }
}
