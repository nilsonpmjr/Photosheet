//! Compositor de referência em CPU.
//!
//! A GPU deve reproduzir este resultado. Canvas, exportação, miniaturas e
//! amostragem passam a ter a mesma definição de composição antes de cada um
//! ganhar uma implementação acelerada.

use crate::core::adjustment::{AdjustmentKind, CurvePoint, LayerAdjustment, LevelsChannel};
use crate::core::blend::LayerBlendMode;
use crate::core::document::Document;
use crate::core::layer::{Layer, LayerKind};
use crate::core::shape::{LayerShapeStyle, ShapeKind};
use crate::core::text::{LayerTextStyle, TextAlignment};
use crate::core::transform::{LayerTransform, Point};
use gtk4::cairo::{Context, FontSlant, FontWeight, Format, ImageSurface};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompositeError {
    InvalidHierarchy,
    SurfaceTooLarge,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompositeSurface {
    pub width: usize,
    pub height: usize,
    /// RGBA8 em alpha reto, no mesmo formato usado pelas camadas raster.
    pub pixels: Vec<u8>,
}

impl CompositeSurface {
    pub fn transparent(width: usize, height: usize) -> Result<Self, CompositeError> {
        let pixel_count = width
            .checked_mul(height)
            .and_then(|count| count.checked_mul(4))
            .ok_or(CompositeError::SurfaceTooLarge)?;
        Ok(Self {
            width,
            height,
            pixels: vec![0; pixel_count],
        })
    }
}

pub struct CpuCompositor;

impl CpuCompositor {
    pub fn render(document: &Document) -> Result<CompositeSurface, CompositeError> {
        let mut output = CompositeSurface::transparent(document.width, document.height)?;
        let layers = ordered_layers(document)?;
        let by_id: HashMap<Uuid, &Layer> = document
            .layers
            .iter()
            .map(|layer| (layer.id, layer))
            .collect();
        for (layer, inherited_opacity) in layers {
            match &layer.kind {
                LayerKind::Pixel {
                    pixels: Some(pixels),
                    width,
                    height,
                    ..
                } => {
                    draw_layer(
                        &mut output,
                        pixels,
                        *width,
                        *height,
                        &layer.transform,
                        layer.mask.as_ref(),
                        inherited_opacity * layer.opacity,
                        layer.blend_mode,
                        layer
                            .clipping_base_id
                            .and_then(|id| by_id.get(&id).copied()),
                        &by_id,
                    );
                }
                LayerKind::Adjustment(adjustment) => {
                    apply_adjustment(&mut output, adjustment, inherited_opacity * layer.opacity);
                }
                LayerKind::Shape(shape) => draw_shape(
                    &mut output,
                    shape,
                    &layer.transform,
                    layer.mask.as_ref(),
                    inherited_opacity * layer.opacity,
                    layer.blend_mode,
                    layer
                        .clipping_base_id
                        .and_then(|id| by_id.get(&id).copied()),
                    &by_id,
                ),
                LayerKind::Text(text) => draw_text(
                    &mut output,
                    text,
                    &layer.transform,
                    inherited_opacity * layer.opacity,
                    layer.blend_mode,
                    layer
                        .clipping_base_id
                        .and_then(|id| by_id.get(&id).copied()),
                    &by_id,
                ),
                _ => {}
            }
        }
        Ok(output)
    }
}

fn draw_text(
    destination: &mut CompositeSurface,
    text: &LayerTextStyle,
    transform: &LayerTransform,
    opacity: f64,
    blend_mode: LayerBlendMode,
    clipping_base: Option<&Layer>,
    by_id: &HashMap<Uuid, &Layer>,
) {
    let width = transform.size.width.ceil().max(1.0) as usize;
    let height = transform.size.height.ceil().max(1.0) as usize;
    let Ok(mut surface) = ImageSurface::create(Format::ARgb32, width as i32, height as i32) else {
        return;
    };
    let Ok(context) = Context::new(&surface) else {
        return;
    };
    context.select_font_face(&text.font_name, FontSlant::Normal, FontWeight::Normal);
    context.set_font_size(text.font_size.max(1.0));
    context.set_source_rgb(
        text.red.clamp(0.0, 1.0),
        text.green.clamp(0.0, 1.0),
        text.blue.clamp(0.0, 1.0),
    );
    let line_height = (text.font_size + text.leading).max(1.0);
    for (index, line) in text.content.lines().enumerate() {
        let x = match text.alignment {
            TextAlignment::Left => 0.0,
            TextAlignment::Center => {
                (width as f64
                    - context
                        .text_extents(line)
                        .map_or(0.0, |extents| extents.width()))
                    / 2.0
            }
            TextAlignment::Right => {
                width as f64
                    - context
                        .text_extents(line)
                        .map_or(0.0, |extents| extents.width())
            }
        };
        context.move_to(x.max(0.0), text.font_size + index as f64 * line_height);
        let _ = context.show_text(line);
    }
    drop(context);
    let stride = surface.stride() as usize;
    let Ok(data) = surface.data() else { return };
    let mut pixels = vec![0; width * height * 4];
    for y in 0..height {
        for x in 0..width {
            let source = &data[y * stride + x * 4..y * stride + x * 4 + 4];
            let target = &mut pixels[(y * width + x) * 4..(y * width + x + 1) * 4];
            let alpha = source[3];
            target[0] = if alpha == 0 {
                0
            } else {
                (source[2] as u16 * 255 / alpha as u16) as u8
            };
            target[1] = if alpha == 0 {
                0
            } else {
                (source[1] as u16 * 255 / alpha as u16) as u8
            };
            target[2] = if alpha == 0 {
                0
            } else {
                (source[0] as u16 * 255 / alpha as u16) as u8
            };
            target[3] = alpha;
        }
    }
    draw_layer(
        destination,
        &pixels,
        width,
        height,
        transform,
        None,
        opacity,
        blend_mode,
        clipping_base,
        by_id,
    );
}

fn apply_adjustment(surface: &mut CompositeSurface, adjustment: &LayerAdjustment, opacity: f64) {
    if opacity <= 0.0 {
        return;
    }
    let opacity = opacity.clamp(0.0, 1.0) as f32;
    if adjustment.kind == AdjustmentKind::GaussianBlur {
        let radius = adjustment.blur_radius.unwrap_or(10.0).clamp(0.1, 250.0) as f32;
        let blurred = gaussian_blur(surface, radius);
        for (destination, source) in surface.pixels.iter_mut().zip(blurred) {
            *destination = (*destination as f32 + (source as f32 - *destination as f32) * opacity)
                .round() as u8;
        }
        return;
    }
    for pixel in surface.pixels.chunks_exact_mut(4) {
        let original = [
            pixel[0] as f32 / 255.0,
            pixel[1] as f32 / 255.0,
            pixel[2] as f32 / 255.0,
        ];
        let Some(adjusted) = adjusted_rgb(adjustment, original) else {
            return;
        };
        for (component, (before, after)) in pixel[..3]
            .iter_mut()
            .zip(original.into_iter().zip(adjusted))
        {
            *component =
                ((before + (after - before) * opacity).clamp(0.0, 1.0) * 255.0).round() as u8;
        }
    }
}

fn gaussian_blur(surface: &CompositeSurface, sigma: f32) -> Vec<u8> {
    let radius = (sigma * 3.0).ceil().min(128.0) as isize;
    let weights: Vec<f32> = (-radius..=radius)
        .map(|offset| (-(offset * offset) as f32 / (2.0 * sigma * sigma)).exp())
        .collect();
    let total: f32 = weights.iter().sum();
    let mut horizontal = vec![0.0_f32; surface.pixels.len()];
    for y in 0..surface.height {
        for x in 0..surface.width {
            for channel in 0..4 {
                for (weight_index, weight) in weights.iter().enumerate() {
                    let sample_x = (x as isize + weight_index as isize - radius)
                        .clamp(0, surface.width.saturating_sub(1) as isize)
                        as usize;
                    horizontal[(y * surface.width + x) * 4 + channel] +=
                        surface.pixels[(y * surface.width + sample_x) * 4 + channel] as f32
                            * weight
                            / total;
                }
            }
        }
    }
    let mut result = vec![0; surface.pixels.len()];
    for y in 0..surface.height {
        for x in 0..surface.width {
            for channel in 0..4 {
                let mut value = 0.0;
                for (weight_index, weight) in weights.iter().enumerate() {
                    let sample_y = (y as isize + weight_index as isize - radius)
                        .clamp(0, surface.height.saturating_sub(1) as isize)
                        as usize;
                    value +=
                        horizontal[(sample_y * surface.width + x) * 4 + channel] * weight / total;
                }
                result[(y * surface.width + x) * 4 + channel] = value.round() as u8;
            }
        }
    }
    result
}

fn adjusted_rgb(adjustment: &LayerAdjustment, color: [f32; 3]) -> Option<[f32; 3]> {
    match adjustment.kind {
        AdjustmentKind::Invert => Some(color.map(|component| 1.0 - component)),
        AdjustmentKind::Levels => Some([
            apply_levels(
                apply_levels(color[0], &adjustment.levels.rgb),
                &adjustment.levels.red,
            ),
            apply_levels(
                apply_levels(color[1], &adjustment.levels.rgb),
                &adjustment.levels.green,
            ),
            apply_levels(
                apply_levels(color[2], &adjustment.levels.rgb),
                &adjustment.levels.blue,
            ),
        ]),
        AdjustmentKind::Curves => Some([
            apply_curve(
                apply_curve(color[0], &adjustment.curves.rgb),
                &adjustment.curves.red,
            ),
            apply_curve(
                apply_curve(color[1], &adjustment.curves.rgb),
                &adjustment.curves.green,
            ),
            apply_curve(
                apply_curve(color[2], &adjustment.curves.rgb),
                &adjustment.curves.blue,
            ),
        ]),
        AdjustmentKind::BlackWhite => {
            let luma = luminosity(color);
            Some([luma; 3])
        }
        AdjustmentKind::GradientMap => {
            let mut luma = luminosity(color);
            if adjustment
                .gradient_map_settings
                .as_ref()
                .is_some_and(|settings| settings.reversed)
            {
                luma = 1.0 - luma;
            }
            Some([luma; 3])
        }
        AdjustmentKind::Exposure => {
            let settings = adjustment.exposure_settings.as_ref()?;
            let gamma = settings.gamma.max(0.01) as f32;
            let multiplier = 2.0_f32.powf(settings.exposure as f32);
            Some(color.map(|component| {
                (component * multiplier + settings.offset as f32)
                    .max(0.0)
                    .powf(1.0 / gamma)
                    .clamp(0.0, 1.0)
            }))
        }
        AdjustmentKind::HueSaturation => {
            let settings = adjustment.hsv_settings.as_ref();
            let hue = settings.map_or(adjustment.hue, |value| value.hue) as f32;
            let saturation =
                settings.map_or(adjustment.saturation, |value| value.saturation) as f32;
            let lightness = settings.map_or(adjustment.lightness, |value| value.lightness) as f32;
            let (hue_base, saturation_base, lightness_base) = rgb_to_hsl(color);
            Some(hsl_to_rgb(
                (hue_base + hue / 360.0).rem_euclid(1.0),
                (saturation_base * (1.0 + saturation / 100.0)).clamp(0.0, 1.0),
                (lightness_base + lightness / 100.0).clamp(0.0, 1.0),
            ))
        }
        _ => None,
    }
}

fn apply_levels(component: f32, levels: &LevelsChannel) -> f32 {
    let input_span = (levels.in_white - levels.in_black).max(0.0001) as f32;
    let normalized = ((component * 255.0 - levels.in_black as f32) / input_span).clamp(0.0, 1.0);
    let gamma = levels.in_gamma.max(0.01) as f32;
    let mapped = normalized.powf(1.0 / gamma);
    ((levels.out_black as f32 + mapped * (levels.out_white - levels.out_black) as f32) / 255.0)
        .clamp(0.0, 1.0)
}

fn apply_curve(component: f32, points: &[CurvePoint]) -> f32 {
    let mut points: Vec<_> = points
        .iter()
        .filter(|point| point.x.is_finite() && point.y.is_finite())
        .collect();
    if points.is_empty() {
        return component;
    }
    points.sort_by(|left, right| left.x.total_cmp(&right.x));
    let component = component.clamp(0.0, 1.0) as f64;
    if component <= points[0].x {
        return points[0].y.clamp(0.0, 1.0) as f32;
    }
    for pair in points.windows(2) {
        let (start, end) = (pair[0], pair[1]);
        if component <= end.x {
            let span = (end.x - start.x).max(f64::EPSILON);
            let amount = ((component - start.x) / span).clamp(0.0, 1.0);
            return (start.y + (end.y - start.y) * amount).clamp(0.0, 1.0) as f32;
        }
    }
    points.last().expect("non-empty curve").y.clamp(0.0, 1.0) as f32
}

fn rgb_to_hsl(color: [f32; 3]) -> (f32, f32, f32) {
    let min = color[0].min(color[1]).min(color[2]);
    let max = color[0].max(color[1]).max(color[2]);
    let lightness = (min + max) / 2.0;
    let delta = max - min;
    if delta == 0.0 {
        return (0.0, 0.0, lightness);
    }
    let saturation = delta / (1.0 - (2.0 * lightness - 1.0).abs());
    let hue = if max == color[0] {
        ((color[1] - color[2]) / delta).rem_euclid(6.0)
    } else if max == color[1] {
        (color[2] - color[0]) / delta + 2.0
    } else {
        (color[0] - color[1]) / delta + 4.0
    } / 6.0;
    (hue, saturation, lightness)
}

fn hsl_to_rgb(hue: f32, saturation: f32, lightness: f32) -> [f32; 3] {
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let hue_sector = hue * 6.0;
    let x = chroma * (1.0 - (hue_sector.rem_euclid(2.0) - 1.0).abs());
    let (red, green, blue) = match hue_sector as u8 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    let offset = lightness - chroma / 2.0;
    [red + offset, green + offset, blue + offset]
}

fn ordered_layers(document: &Document) -> Result<Vec<(&Layer, f64)>, CompositeError> {
    let by_id: HashMap<Uuid, &Layer> = document
        .layers
        .iter()
        .map(|layer| (layer.id, layer))
        .collect();
    if by_id.len() != document.layers.len() {
        return Err(CompositeError::InvalidHierarchy);
    }
    for layer in &document.layers {
        let mut seen = HashSet::from([layer.id]);
        let mut parent = layer.parent_id;
        while let Some(id) = parent {
            let Some(parent_layer) = by_id.get(&id) else {
                return Err(CompositeError::InvalidHierarchy);
            };
            if seen.len() > 64 || !seen.insert(id) || !parent_layer.is_group {
                return Err(CompositeError::InvalidHierarchy);
            }
            parent = parent_layer.parent_id;
        }
    }

    let mut result = Vec::new();
    visit_children(None, 0, 1.0, true, &document.layers, &by_id, &mut result)?;
    if result.len()
        != document
            .layers
            .iter()
            .filter(|layer| !layer.is_group)
            .count()
    {
        return Err(CompositeError::InvalidHierarchy);
    }
    Ok(result)
}

fn visit_children<'a>(
    parent: Option<Uuid>,
    depth: usize,
    inherited_opacity: f64,
    parent_visible: bool,
    all_layers: &'a [Layer],
    by_id: &HashMap<Uuid, &'a Layer>,
    result: &mut Vec<(&'a Layer, f64)>,
) -> Result<(), CompositeError> {
    if depth > 64 {
        return Err(CompositeError::InvalidHierarchy);
    }
    for layer in all_layers.iter().filter(|layer| layer.parent_id == parent) {
        if let Some(parent_id) = layer.parent_id {
            let Some(parent_layer) = by_id.get(&parent_id) else {
                return Err(CompositeError::InvalidHierarchy);
            };
            if !parent_layer.is_group {
                return Err(CompositeError::InvalidHierarchy);
            }
        }
        let visible = parent_visible && layer.is_visible;
        if layer.is_group {
            visit_children(
                Some(layer.id),
                depth + 1,
                inherited_opacity * layer.opacity,
                visible,
                all_layers,
                by_id,
                result,
            )?;
        } else if visible {
            result.push((layer, inherited_opacity));
        }
    }
    Ok(())
}

fn draw_layer(
    destination: &mut CompositeSurface,
    source: &[u8],
    source_width: usize,
    source_height: usize,
    transform: &LayerTransform,
    mask: Option<&crate::core::mask::LayerMask>,
    opacity: f64,
    blend_mode: LayerBlendMode,
    clipping_base: Option<&Layer>,
    by_id: &HashMap<Uuid, &Layer>,
) {
    if source.len() != source_width.saturating_mul(source_height).saturating_mul(4)
        || opacity <= 0.0
    {
        return;
    }
    let opacity = opacity.clamp(0.0, 1.0) as f32;
    for y in 0..destination.height {
        for x in 0..destination.width {
            let Some((source_x, source_y)) = source_coordinate(
                transform,
                source_width,
                source_height,
                x as f64 + 0.5,
                y as f64 + 0.5,
            ) else {
                continue;
            };
            let source_index = (source_y * source_width + source_x) * 4;
            let mut source_alpha = source[source_index + 3] as f32 / 255.0 * opacity;
            if let Some(mask) = mask.filter(|mask| mask.enabled) {
                source_alpha *=
                    sample_mask(mask, source_width, source_height, source_x, source_y, x, y);
            }
            if let Some(clipping_base) = clipping_base {
                source_alpha *= layer_alpha_at(clipping_base, x, y, by_id, &mut HashSet::new());
            }
            if source_alpha <= 0.0 {
                continue;
            }
            let source_rgb = [
                source[source_index] as f32 / 255.0,
                source[source_index + 1] as f32 / 255.0,
                source[source_index + 2] as f32 / 255.0,
            ];
            let destination_index = (y * destination.width + x) * 4;
            composite_pixel(
                &mut destination.pixels[destination_index..destination_index + 4],
                source_rgb,
                source_alpha,
                blend_mode,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_shape(
    destination: &mut CompositeSurface,
    shape: &LayerShapeStyle,
    transform: &LayerTransform,
    mask: Option<&crate::core::mask::LayerMask>,
    opacity: f64,
    blend_mode: LayerBlendMode,
    clipping_base: Option<&Layer>,
    by_id: &HashMap<Uuid, &Layer>,
) {
    if opacity <= 0.0 {
        return;
    }
    let color = [
        shape.red.clamp(0.0, 1.0) as f32,
        shape.green.clamp(0.0, 1.0) as f32,
        shape.blue.clamp(0.0, 1.0) as f32,
    ];
    for y in 0..destination.height {
        for x in 0..destination.width {
            let Some((local_x, local_y)) =
                normalized_coordinate(transform, x as f64 + 0.5, y as f64 + 0.5)
            else {
                continue;
            };
            if !shape_contains(
                shape,
                local_x,
                local_y,
                transform.size.width,
                transform.size.height,
            ) {
                continue;
            }
            let mut alpha = opacity.clamp(0.0, 1.0) as f32;
            if let Some(mask) = mask.filter(|mask| mask.enabled) {
                let source_x = (local_x * mask.width as f64).floor() as usize;
                let source_y = (local_y * mask.height as f64).floor() as usize;
                alpha *= sample_mask(
                    mask,
                    mask.width,
                    mask.height,
                    source_x.min(mask.width - 1),
                    source_y.min(mask.height - 1),
                    x,
                    y,
                );
            }
            if let Some(base) = clipping_base {
                alpha *= layer_alpha_at(base, x, y, by_id, &mut HashSet::new());
            }
            let index = (y * destination.width + x) * 4;
            composite_pixel(
                &mut destination.pixels[index..index + 4],
                color,
                alpha,
                blend_mode,
            );
        }
    }
}

fn shape_contains(shape: &LayerShapeStyle, x: f64, y: f64, width: f64, height: f64) -> bool {
    match shape.kind {
        ShapeKind::Rectangle => true,
        ShapeKind::Ellipse => (x - 0.5).powi(2) + (y - 0.5).powi(2) <= 0.25,
        ShapeKind::RoundedRectangle => {
            let radius = (shape.corner_radius.max(0.0) / width.min(height).max(1.0)).min(0.5);
            if x >= radius && x <= 1.0 - radius || y >= radius && y <= 1.0 - radius {
                true
            } else {
                let cx = if x < 0.5 { radius } else { 1.0 - radius };
                let cy = if y < 0.5 { radius } else { 1.0 - radius };
                (x - cx).powi(2) + (y - cy).powi(2) <= radius.powi(2)
            }
        }
        ShapeKind::Line => {
            let start = shape.start.unwrap_or(Point { x: 0.0, y: 0.0 });
            let end = shape.end.unwrap_or(Point { x: 1.0, y: 1.0 });
            let dx = end.x - start.x;
            let dy = end.y - start.y;
            let length_squared = dx * dx + dy * dy;
            let t = if length_squared > 0.0 {
                (((x - start.x) * dx + (y - start.y) * dy) / length_squared).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let distance =
                ((x - (start.x + t * dx)).powi(2) + (y - (start.y + t * dy)).powi(2)).sqrt();
            distance <= shape.line_width.unwrap_or(1.0) / width.min(height).max(1.0) / 2.0
        }
    }
}

fn layer_alpha_at(
    layer: &Layer,
    canvas_x: usize,
    canvas_y: usize,
    by_id: &HashMap<Uuid, &Layer>,
    visiting: &mut HashSet<Uuid>,
) -> f32 {
    if !visiting.insert(layer.id) {
        return 0.0;
    }
    let result = match &layer.kind {
        LayerKind::Pixel {
            pixels: Some(pixels),
            width,
            height,
            ..
        } => {
            let Some((source_x, source_y)) = source_coordinate(
                &layer.transform,
                *width,
                *height,
                canvas_x as f64 + 0.5,
                canvas_y as f64 + 0.5,
            ) else {
                visiting.remove(&layer.id);
                return 0.0;
            };
            let mut alpha = pixels[(source_y * *width + source_x) * 4 + 3] as f32 / 255.0;
            alpha *= effective_opacity(layer, by_id) as f32;
            if let Some(mask) = layer.mask.as_ref().filter(|mask| mask.enabled) {
                alpha *= sample_mask(
                    mask, *width, *height, source_x, source_y, canvas_x, canvas_y,
                );
            }
            if let Some(base_id) = layer.clipping_base_id {
                alpha *= by_id
                    .get(&base_id)
                    .map(|base| layer_alpha_at(base, canvas_x, canvas_y, by_id, visiting))
                    .unwrap_or(0.0);
            }
            alpha
        }
        _ => 0.0,
    };
    visiting.remove(&layer.id);
    result
}

fn effective_opacity(layer: &Layer, by_id: &HashMap<Uuid, &Layer>) -> f64 {
    let mut opacity = layer.opacity;
    let mut parent = layer.parent_id;
    let mut depth = 0;
    while let Some(id) = parent {
        let Some(group) = by_id.get(&id) else {
            return 0.0;
        };
        opacity *= group.opacity;
        parent = group.parent_id;
        depth += 1;
        if depth > 64 {
            return 0.0;
        }
    }
    opacity.clamp(0.0, 1.0)
}

fn source_coordinate(
    transform: &LayerTransform,
    width: usize,
    height: usize,
    canvas_x: f64,
    canvas_y: f64,
) -> Option<(usize, usize)> {
    let (normalized_x, normalized_y) = normalized_coordinate(transform, canvas_x, canvas_y)?;
    let source_x = (normalized_x * width as f64).floor() as usize;
    let source_y = (normalized_y * height as f64).floor() as usize;
    (source_x < width && source_y < height).then_some((source_x, source_y))
}

fn normalized_coordinate(
    transform: &LayerTransform,
    canvas_x: f64,
    canvas_y: f64,
) -> Option<(f64, f64)> {
    let center = transform.center();
    let angle = -transform.radians();
    let dx = canvas_x - center.x;
    let dy = canvas_y - center.y;
    let local_x = dx * angle.cos() - dy * angle.sin() + transform.size.width / 2.0;
    let local_y = dx * angle.sin() + dy * angle.cos() + transform.size.height / 2.0;
    if local_x < 0.0
        || local_y < 0.0
        || local_x >= transform.size.width
        || local_y >= transform.size.height
    {
        return None;
    }
    let normalized_x = local_x / transform.size.width;
    let normalized_y = local_y / transform.size.height;
    let normalized_x = if transform.flip_x {
        1.0 - normalized_x
    } else {
        normalized_x
    };
    let normalized_y = if transform.flip_y {
        1.0 - normalized_y
    } else {
        normalized_y
    };
    Some((normalized_x, normalized_y))
}

fn sample_mask(
    mask: &crate::core::mask::LayerMask,
    source_width: usize,
    source_height: usize,
    source_x: usize,
    source_y: usize,
    canvas_x: usize,
    canvas_y: usize,
) -> f32 {
    if mask.data.len() != mask.width.saturating_mul(mask.height)
        || mask.width == 0
        || mask.height == 0
    {
        return 0.0;
    }
    let (x, y) = if mask.linked {
        let x = source_x * mask.width / source_width;
        let y = source_y * mask.height / source_height;
        (x.min(mask.width - 1), y.min(mask.height - 1))
    } else if let Some(placement) = &mask.placement {
        let Some((x, y)) = source_coordinate(
            placement,
            mask.width,
            mask.height,
            canvas_x as f64 + 0.5,
            canvas_y as f64 + 0.5,
        ) else {
            return 0.0;
        };
        (x, y)
    } else {
        return 0.0;
    };
    mask.data[y * mask.width + x] as f32 / 255.0
}

fn composite_pixel(
    destination: &mut [u8],
    source_rgb: [f32; 3],
    source_alpha: f32,
    blend_mode: LayerBlendMode,
) {
    let destination_alpha = destination[3] as f32 / 255.0;
    let destination_rgb = [
        destination[0] as f32 / 255.0,
        destination[1] as f32 / 255.0,
        destination[2] as f32 / 255.0,
    ];
    let blended = blend_rgb(blend_mode, source_rgb, destination_rgb);
    let output_alpha = source_alpha + destination_alpha * (1.0 - source_alpha);
    let output_rgb = if output_alpha > 0.0 {
        [
            ((1.0 - source_alpha) * destination_alpha * destination_rgb[0]
                + (1.0 - destination_alpha) * source_alpha * source_rgb[0]
                + source_alpha * destination_alpha * blended[0])
                / output_alpha,
            ((1.0 - source_alpha) * destination_alpha * destination_rgb[1]
                + (1.0 - destination_alpha) * source_alpha * source_rgb[1]
                + source_alpha * destination_alpha * blended[1])
                / output_alpha,
            ((1.0 - source_alpha) * destination_alpha * destination_rgb[2]
                + (1.0 - destination_alpha) * source_alpha * source_rgb[2]
                + source_alpha * destination_alpha * blended[2])
                / output_alpha,
        ]
    } else {
        [0.0; 3]
    };
    destination[0] = (output_rgb[0].clamp(0.0, 1.0) * 255.0).round() as u8;
    destination[1] = (output_rgb[1].clamp(0.0, 1.0) * 255.0).round() as u8;
    destination[2] = (output_rgb[2].clamp(0.0, 1.0) * 255.0).round() as u8;
    destination[3] = (output_alpha * 255.0).round() as u8;
}

fn blend_rgb(mode: LayerBlendMode, source: [f32; 3], destination: [f32; 3]) -> [f32; 3] {
    match mode {
        LayerBlendMode::Hue => set_luminosity(
            set_saturation(source, saturation(destination)),
            luminosity(destination),
        ),
        LayerBlendMode::Saturation => set_luminosity(
            set_saturation(destination, saturation(source)),
            luminosity(destination),
        ),
        LayerBlendMode::Color => set_luminosity(source, luminosity(destination)),
        LayerBlendMode::Luminosity => set_luminosity(destination, luminosity(source)),
        _ => [
            mode.blend_channel(source[0], destination[0]),
            mode.blend_channel(source[1], destination[1]),
            mode.blend_channel(source[2], destination[2]),
        ],
    }
}

fn luminosity(color: [f32; 3]) -> f32 {
    0.3 * color[0] + 0.59 * color[1] + 0.11 * color[2]
}

fn saturation(color: [f32; 3]) -> f32 {
    let min = color[0].min(color[1]).min(color[2]);
    let max = color[0].max(color[1]).max(color[2]);
    max - min
}

fn set_luminosity(color: [f32; 3], target: f32) -> [f32; 3] {
    clip_color(color.map(|component| component + target - luminosity(color)))
}

fn clip_color(mut color: [f32; 3]) -> [f32; 3] {
    let lum = luminosity(color);
    let min = color[0].min(color[1]).min(color[2]);
    let max = color[0].max(color[1]).max(color[2]);
    if min < 0.0 {
        color = color.map(|component| lum + (component - lum) * lum / (lum - min));
    }
    if max > 1.0 {
        color = color.map(|component| lum + (component - lum) * (1.0 - lum) / (max - lum));
    }
    color
}

fn set_saturation(color: [f32; 3], target: f32) -> [f32; 3] {
    let mut order = [(color[0], 0), (color[1], 1), (color[2], 2)];
    order.sort_by(|left, right| left.0.total_cmp(&right.0));
    let (min, min_index) = order[0];
    let (mid, mid_index) = order[1];
    let (max, max_index) = order[2];
    let mut result = [0.0; 3];
    if max > min {
        result[mid_index] = (mid - min) * target / (max - min);
        result[max_index] = target;
    }
    result[min_index] = 0.0;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::layer::Layer;
    use crate::core::mask::LayerMask;
    use crate::core::transform::{LayerTransform, Size};

    fn pixel_layer(name: &str, pixels: Vec<u8>) -> Layer {
        let mut layer = Layer::new_pixel(
            name.to_string(),
            1,
            1,
            LayerTransform::new(Point::ZERO, Size::new(1.0, 1.0)),
        );
        let LayerKind::Pixel { pixels: buffer, .. } = &mut layer.kind else {
            unreachable!();
        };
        *buffer = Some(pixels);
        layer
    }

    fn shape_layer(kind: ShapeKind, width: usize, height: usize) -> Layer {
        let mut layer = Layer::new_pixel(
            "shape".to_string(),
            width,
            height,
            LayerTransform::new(Point::ZERO, Size::new(width as f64, height as f64)),
        );
        layer.kind = LayerKind::Shape(LayerShapeStyle {
            kind,
            red: 1.0,
            green: 0.0,
            blue: 0.0,
            corner_radius: 1.0,
            line_width: None,
            start: None,
            end: None,
        });
        layer
    }

    #[test]
    fn composites_alpha_and_group_opacity() {
        let mut document = Document::new(1, 1, 72.0);
        document.add_layer(pixel_layer("bottom", vec![0, 0, 255, 255]));

        let mut group = Layer::new_group(
            "group".to_string(),
            LayerTransform::new(Point::ZERO, Size::new(1.0, 1.0)),
        );
        group.opacity = 0.5;
        let group_id = group.id;
        document.add_layer(group);
        let mut top = pixel_layer("top", vec![255, 0, 0, 255]);
        top.parent_id = Some(group_id);
        document.add_layer(top);

        let surface = CpuCompositor::render(&document).unwrap();
        assert_eq!(surface.pixels, vec![128, 0, 128, 255]);
    }

    #[test]
    fn applies_layer_mask() {
        let mut document = Document::new(1, 1, 72.0);
        let mut layer = pixel_layer("masked", vec![255, 0, 0, 255]);
        layer.mask = Some(LayerMask::new_uniform(1, 1, 128));
        document.add_layer(layer);

        let surface = CpuCompositor::render(&document).unwrap();
        assert_eq!(surface.pixels, vec![255, 0, 0, 128]);
    }

    #[test]
    fn applies_clipping_mask_alpha() {
        let mut document = Document::new(1, 1, 72.0);
        document.add_layer(pixel_layer("base", vec![0, 0, 0, 128]));
        let base_id = document.active_layer_id.unwrap();
        let mut clipped = pixel_layer("clipped", vec![255, 0, 0, 255]);
        clipped.clipping_base_id = Some(base_id);
        document.add_layer(clipped);

        let surface = CpuCompositor::render(&document).unwrap();
        assert_eq!(surface.pixels, vec![170, 0, 0, 192]);
    }

    #[test]
    fn applies_horizontal_flip_transform() {
        let mut document = Document::new(2, 1, 72.0);
        let mut layer = Layer::new_pixel(
            "flipped".to_string(),
            2,
            1,
            LayerTransform::new(Point::ZERO, Size::new(2.0, 1.0)),
        );
        layer.transform.flip_x = true;
        let LayerKind::Pixel { pixels, .. } = &mut layer.kind else {
            unreachable!();
        };
        *pixels = Some(vec![255, 0, 0, 255, 0, 0, 255, 255]);
        document.add_layer(layer);

        let surface = CpuCompositor::render(&document).unwrap();
        assert_eq!(surface.pixels, vec![0, 0, 255, 255, 255, 0, 0, 255]);
    }

    #[test]
    fn applies_clockwise_rotation_around_layer_center() {
        let mut document = Document::new(2, 2, 72.0);
        let mut layer = Layer::new_pixel(
            "rotated".to_string(),
            2,
            2,
            LayerTransform::new(Point::ZERO, Size::new(2.0, 2.0)),
        );
        layer.transform.rotation = 90.0;
        let LayerKind::Pixel { pixels, .. } = &mut layer.kind else {
            unreachable!();
        };
        *pixels = Some(vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ]);
        document.add_layer(layer);

        let surface = CpuCompositor::render(&document).unwrap();
        assert_eq!(
            surface.pixels,
            vec![0, 0, 255, 255, 255, 0, 0, 255, 255, 255, 255, 255, 0, 255, 0, 255,]
        );
    }

    #[test]
    fn applies_non_separable_color_blend() {
        let mut document = Document::new(1, 1, 72.0);
        document.add_layer(pixel_layer("bottom", vec![128, 128, 128, 255]));
        let mut top = pixel_layer("top", vec![255, 0, 0, 255]);
        top.blend_mode = LayerBlendMode::Color;
        document.add_layer(top);

        let surface = CpuCompositor::render(&document).unwrap();
        assert_eq!(surface.pixels, vec![255, 74, 74, 255]);
    }

    #[test]
    fn applies_every_separable_blend_mode() {
        let source = [204, 51, 153, 255];
        let backdrop = [51, 178, 102, 255];
        for mode in LayerBlendMode::all() {
            if matches!(
                mode,
                LayerBlendMode::Hue
                    | LayerBlendMode::Saturation
                    | LayerBlendMode::Color
                    | LayerBlendMode::Luminosity
            ) {
                continue;
            }
            let mut document = Document::new(1, 1, 72.0);
            document.add_layer(pixel_layer("bottom", backdrop.to_vec()));
            let mut top = pixel_layer("top", source.to_vec());
            top.blend_mode = *mode;
            document.add_layer(top);

            let surface = CpuCompositor::render(&document).unwrap();
            let expected = [
                (mode.blend_channel(source[0] as f32 / 255.0, backdrop[0] as f32 / 255.0) * 255.0)
                    .round() as u8,
                (mode.blend_channel(source[1] as f32 / 255.0, backdrop[1] as f32 / 255.0) * 255.0)
                    .round() as u8,
                (mode.blend_channel(source[2] as f32 / 255.0, backdrop[2] as f32 / 255.0) * 255.0)
                    .round() as u8,
                255,
            ];
            assert_eq!(surface.pixels, expected, "{}", mode.name());
        }
    }

    #[test]
    fn component_blends_preserve_their_documented_components() {
        let source = [0.8, 0.2, 0.6];
        let destination = [0.2, 0.6, 0.4];

        let hue = blend_rgb(LayerBlendMode::Hue, source, destination);
        assert!((luminosity(hue) - luminosity(destination)).abs() < 0.0001);
        assert!((saturation(hue) - saturation(destination)).abs() < 0.0001);

        let saturation_mode = blend_rgb(LayerBlendMode::Saturation, source, destination);
        assert!((luminosity(saturation_mode) - luminosity(destination)).abs() < 0.0001);
        assert!((saturation(saturation_mode) - saturation(source)).abs() < 0.0001);

        let color = blend_rgb(LayerBlendMode::Color, source, destination);
        assert!((luminosity(color) - luminosity(destination)).abs() < 0.0001);
        assert!((saturation(color) - saturation(source)).abs() < 0.0001);

        let luminosity_mode = blend_rgb(LayerBlendMode::Luminosity, source, destination);
        assert!((luminosity(luminosity_mode) - luminosity(source)).abs() < 0.0001);
        assert!((saturation(luminosity_mode) - saturation(destination)).abs() < 0.0001);
    }

    #[test]
    fn applies_invert_adjustment_with_layer_opacity() {
        let mut document = Document::new(1, 1, 72.0);
        document.add_layer(pixel_layer("bottom", vec![64, 128, 192, 255]));
        let mut adjustment = Layer::new_adjustment(
            "invert".to_string(),
            LayerAdjustment::invert(),
            LayerTransform::new(Point::ZERO, Size::new(1.0, 1.0)),
        );
        adjustment.opacity = 0.5;
        document.add_layer(adjustment);

        let surface = CpuCompositor::render(&document).unwrap();
        assert_eq!(surface.pixels, vec![128, 128, 128, 255]);
    }

    #[test]
    fn applies_basic_color_adjustments() {
        let hue = adjusted_rgb(
            &LayerAdjustment::hue_saturation(120.0, 0.0, 0.0),
            [1.0, 0.0, 0.0],
        )
        .unwrap();
        assert!((hue[0] - 0.0).abs() < 0.0001);
        assert!((hue[1] - 1.0).abs() < 0.0001);
        assert!((hue[2] - 0.0).abs() < 0.0001);

        let exposure = adjusted_rgb(&LayerAdjustment::exposure(1.0, 0.0, 1.0), [0.25; 3]).unwrap();
        assert_eq!(exposure, [0.5; 3]);

        let levels = adjusted_rgb(
            &LayerAdjustment::levels(0.0, 2.0, 255.0, 0.0, 255.0),
            [0.25; 3],
        )
        .unwrap();
        assert_eq!(levels, [0.5; 3]);

        let mut curves = LayerAdjustment::curves();
        curves.curves.rgb = vec![
            CurvePoint { x: 0.0, y: 0.0 },
            CurvePoint { x: 0.5, y: 0.25 },
            CurvePoint { x: 1.0, y: 1.0 },
        ];
        let curved = adjusted_rgb(&curves, [0.5; 3]).unwrap();
        assert_eq!(curved, [0.25; 3]);

        let black_and_white =
            adjusted_rgb(&LayerAdjustment::black_and_white(), [1.0, 0.0, 0.0]).unwrap();
        assert_eq!(black_and_white, [0.3; 3]);

        let mut gradient_map = LayerAdjustment::new(AdjustmentKind::GradientMap);
        gradient_map.gradient_map_settings = Some(Default::default());
        assert_eq!(
            adjusted_rgb(&gradient_map, [1.0, 0.0, 0.0]).unwrap(),
            [0.3; 3]
        );
        gradient_map
            .gradient_map_settings
            .as_mut()
            .unwrap()
            .reversed = true;
        assert_eq!(
            adjusted_rgb(&gradient_map, [1.0, 0.0, 0.0]).unwrap(),
            [0.7; 3]
        );
    }

    #[test]
    fn renders_vector_rectangle_and_ellipse_layers() {
        let mut rectangle_document = Document::new(2, 2, 72.0);
        rectangle_document.add_layer(shape_layer(ShapeKind::Rectangle, 2, 2));
        assert_eq!(
            CpuCompositor::render(&rectangle_document).unwrap().pixels,
            vec![255, 0, 0, 255].repeat(4)
        );

        let mut ellipse_document = Document::new(4, 4, 72.0);
        ellipse_document.add_layer(shape_layer(ShapeKind::Ellipse, 4, 4));
        let surface = CpuCompositor::render(&ellipse_document).unwrap();
        assert_eq!(&surface.pixels[0..4], &[0, 0, 0, 0]);
        assert_eq!(&surface.pixels[20..24], &[255, 0, 0, 255]);
    }

    #[test]
    fn renders_rounded_rectangle_and_line_layers() {
        let mut rounded_document = Document::new(4, 4, 72.0);
        let mut rounded = shape_layer(ShapeKind::RoundedRectangle, 4, 4);
        if let LayerKind::Shape(style) = &mut rounded.kind {
            style.corner_radius = 2.0;
        }
        rounded_document.add_layer(rounded);
        assert_eq!(
            &CpuCompositor::render(&rounded_document).unwrap().pixels[0..4],
            &[0, 0, 0, 0]
        );

        let mut line_document = Document::new(4, 4, 72.0);
        let mut line = shape_layer(ShapeKind::Line, 4, 4);
        if let LayerKind::Shape(style) = &mut line.kind {
            style.line_width = Some(1.0);
            style.start = Some(Point { x: 0.0, y: 0.0 });
            style.end = Some(Point { x: 1.0, y: 1.0 });
        }
        line_document.add_layer(line);
        let surface = CpuCompositor::render(&line_document).unwrap();
        assert_eq!(&surface.pixels[0..4], &[255, 0, 0, 255]);
        assert_eq!(&surface.pixels[12..16], &[0, 0, 0, 0]);
    }

    #[test]
    fn renders_text_layer_into_composite_surface() {
        let mut document = Document::new(48, 32, 72.0);
        let mut layer = Layer::new_pixel(
            "text".to_string(),
            1,
            1,
            LayerTransform::new(Point::ZERO, Size::new(48.0, 32.0)),
        );
        layer.kind = LayerKind::Text(LayerTextStyle {
            content: "A".to_string(),
            font_size: 24.0,
            red: 1.0,
            green: 0.0,
            blue: 0.0,
            ..Default::default()
        });
        document.add_layer(layer);

        let surface = CpuCompositor::render(&document).unwrap();
        assert!(surface
            .pixels
            .chunks_exact(4)
            .any(|pixel| pixel[0] > 0 && pixel[3] > 0));
    }

    #[test]
    fn applies_gaussian_blur_adjustment() {
        let mut document = Document::new(3, 1, 72.0);
        let mut pixels = Layer::new_pixel(
            "pixels".to_string(),
            3,
            1,
            LayerTransform::new(Point::ZERO, Size::new(3.0, 1.0)),
        );
        if let LayerKind::Pixel { pixels: buffer, .. } = &mut pixels.kind {
            *buffer = Some(vec![0, 0, 0, 255, 255, 0, 0, 255, 0, 0, 0, 255]);
        }
        document.add_layer(pixels);
        let adjustment = Layer::new_adjustment(
            "blur".to_string(),
            LayerAdjustment::gaussian_blur(1.0),
            LayerTransform::new(Point::ZERO, Size::new(3.0, 1.0)),
        );
        document.add_layer(adjustment);

        let surface = CpuCompositor::render(&document).unwrap();
        assert!(surface.pixels[0] > 0);
        assert!(surface.pixels[4] < 255 && surface.pixels[4] > surface.pixels[0]);
        assert_eq!(surface.pixels[3], 255);
    }
}
