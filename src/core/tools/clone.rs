//! Carimbo de Clonagem (Clone Stamp Tool).
//! Traduzido de Compositor/Document/EditorSession+Brush.swift.

use crate::core::session::BrushSettings;

pub struct CloneStampEngine;

impl CloneStampEngine {
    /// Aplica um dab do carimbo copiando pixels da coordenada de origem com o deslocamento (offset).
    pub fn render_clone_dab(
        dest_cx: f64,
        dest_cy: f64,
        offset_x: f64,
        offset_y: f64,
        settings: &BrushSettings,
        source_pixels: &[u8],
        target_pixels: &mut [u8],
        width: usize,
        height: usize,
    ) {
        let radius = (settings.size / 2.0).max(0.5);
        let hardness = settings.hardness.clamp(0.0, 0.99);
        let opacity = settings.opacity.clamp(0.0, 1.0) * settings.flow.clamp(0.0, 1.0);

        let min_x = ((dest_cx - radius).floor().max(0.0)) as usize;
        let max_x = ((dest_cx + radius).ceil().min((width - 1) as f64)) as usize;
        let min_y = ((dest_cy - radius).floor().max(0.0)) as usize;
        let max_y = ((dest_cy + radius).ceil().min((height - 1) as f64)) as usize;

        for py in min_y..=max_y {
            let dy = py as f64 - dest_cy;
            let dy2 = dy * dy;

            for px in min_x..=max_x {
                let dx = px as f64 - dest_cx;
                let dist = (dx * dx + dy2).sqrt();

                if dist > radius {
                    continue;
                }

                // Coordenada correspondente na fonte
                let src_px = px as f64 + offset_x;
                let src_py = py as f64 + offset_y;

                if src_px < 0.0 || src_px >= width as f64 || src_py < 0.0 || src_py >= height as f64 {
                    continue;
                }

                let src_idx = (src_py as usize * width + src_px as usize) * 4;
                let sample_color = [
                    source_pixels[src_idx],
                    source_pixels[src_idx + 1],
                    source_pixels[src_idx + 2],
                    source_pixels[src_idx + 3],
                ];

                let t = dist / radius;
                let dab_alpha = if t <= hardness {
                    1.0
                } else {
                    ((1.0 - t) / (1.0 - hardness)).clamp(0.0, 1.0)
                };

                let effective_alpha = dab_alpha * opacity;
                if effective_alpha <= 0.001 {
                    continue;
                }

                let dst_idx = (py * width + px) * 4;
                let src_r = sample_color[0] as f64 / 255.0;
                let src_g = sample_color[1] as f64 / 255.0;
                let src_b = sample_color[2] as f64 / 255.0;
                let src_a = (sample_color[3] as f64 / 255.0) * effective_alpha;

                let dst_r = target_pixels[dst_idx] as f64 / 255.0;
                let dst_g = target_pixels[dst_idx + 1] as f64 / 255.0;
                let dst_b = target_pixels[dst_idx + 2] as f64 / 255.0;
                let dst_a = target_pixels[dst_idx + 3] as f64 / 255.0;

                let out_a = src_a + dst_a * (1.0 - src_a);
                if out_a > 0.0 {
                    let out_r = (src_r * src_a + dst_r * dst_a * (1.0 - src_a)) / out_a;
                    let out_g = (src_g * src_a + dst_g * dst_a * (1.0 - src_a)) / out_a;
                    let out_b = (src_b * src_a + dst_b * dst_a * (1.0 - src_a)) / out_a;

                    target_pixels[dst_idx] = (out_r * 255.0).clamp(0.0, 255.0).round() as u8;
                    target_pixels[dst_idx + 1] = (out_g * 255.0).clamp(0.0, 255.0).round() as u8;
                    target_pixels[dst_idx + 2] = (out_b * 255.0).clamp(0.0, 255.0).round() as u8;
                    target_pixels[dst_idx + 3] = (out_a * 255.0).clamp(0.0, 255.0).round() as u8;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clone_stamp_copy() {
        let width = 40;
        let height = 40;
        let mut source = vec![0u8; width * height * 4];
        let mut target = vec![0u8; width * height * 4];

        // Preenche ponto de origem (10, 10) com cor verde
        let src_idx = (10 * width + 10) * 4;
        source[src_idx] = 0;
        source[src_idx + 1] = 255;
        source[src_idx + 2] = 0;
        source[src_idx + 3] = 255;

        // Queremos copiar de (10, 10) para (20, 20)
        // offset = src - dest = 10 - 20 = -10
        let mut settings = BrushSettings::default();
        settings.size = 6.0;
        settings.hardness = 1.0;
        settings.opacity = 1.0;

        CloneStampEngine::render_clone_dab(
            20.0, 20.0, -10.0, -10.0, &settings, &source, &mut target, width, height,
        );

        let dst_idx = (20 * width + 20) * 4;
        assert_eq!(target[dst_idx + 1], 255); // Canal verde clonado com sucesso
    }
}
