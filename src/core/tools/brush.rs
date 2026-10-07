//! Mecanismo de Traçado de Pincel e Borracha (Brush & Eraser).
//! Traduzido de Compositor/Document/EditorSession+Brush.swift e BrushPixels.c.

use crate::core::selection::DocumentSelection;
use crate::core::session::BrushSettings;

pub struct BrushEngine;

impl BrushEngine {
    /// Aplica um único "dab" (marca circular) do pincel na camada de pixels.
    pub fn render_dab(
        cx: f64,
        cy: f64,
        settings: &BrushSettings,
        color: [u8; 4],
        is_eraser: bool,
        pixels: &mut [u8],
        width: usize,
        height: usize,
        selection: Option<&DocumentSelection>,
    ) {
        let radius = (settings.size / 2.0).max(0.5);
        let hardness = settings.hardness.clamp(0.0, 0.99);
        let opacity = settings.opacity.clamp(0.0, 1.0) * settings.flow.clamp(0.0, 1.0);

        let min_x = ((cx - radius).floor().max(0.0)) as usize;
        let max_x = ((cx + radius).ceil().min((width - 1) as f64)) as usize;
        let min_y = ((cy - radius).floor().max(0.0)) as usize;
        let max_y = ((cy + radius).ceil().min((height - 1) as f64)) as usize;

        for py in min_y..=max_y {
            let dy = py as f64 - cy;
            let dy2 = dy * dy;

            for px in min_x..=max_x {
                let dx = px as f64 - cx;
                let dist = (dx * dx + dy2).sqrt();

                if dist > radius {
                    continue;
                }

                // Cálculo da atenuação por dureza (falloff)
                let t = dist / radius;
                let dab_alpha = if t <= hardness {
                    1.0
                } else {
                    ((1.0 - t) / (1.0 - hardness)).clamp(0.0, 1.0)
                };

                // Modulação por seleção (se houver seleção ativa)
                let sel_factor = if let Some(sel) = selection {
                    sel.get_value(px, py) as f64 / 255.0
                } else {
                    1.0
                };

                let effective_alpha = dab_alpha * opacity * sel_factor;
                if effective_alpha <= 0.001 {
                    continue;
                }

                let idx = (py * width + px) * 4;
                if is_eraser {
                    // Modo Borracha: reduz o canal alfa
                    let current_a = pixels[idx + 3] as f64 / 255.0;
                    let new_a = (current_a * (1.0 - effective_alpha)).max(0.0);
                    pixels[idx + 3] = (new_a * 255.0).round() as u8;
                } else {
                    // Modo Pintura: mesclagem alfa padrão
                    let src_r = color[0] as f64 / 255.0;
                    let src_g = color[1] as f64 / 255.0;
                    let src_b = color[2] as f64 / 255.0;
                    let src_a = (color[3] as f64 / 255.0) * effective_alpha;

                    let dst_r = pixels[idx] as f64 / 255.0;
                    let dst_g = pixels[idx + 1] as f64 / 255.0;
                    let dst_b = pixels[idx + 2] as f64 / 255.0;
                    let dst_a = pixels[idx + 3] as f64 / 255.0;

                    let out_a = src_a + dst_a * (1.0 - src_a);
                    if out_a > 0.0 {
                        let out_r = (src_r * src_a + dst_r * dst_a * (1.0 - src_a)) / out_a;
                        let out_g = (src_g * src_a + dst_g * dst_a * (1.0 - src_a)) / out_a;
                        let out_b = (src_b * src_a + dst_b * dst_a * (1.0 - src_a)) / out_a;

                        pixels[idx] = (out_r * 255.0).clamp(0.0, 255.0).round() as u8;
                        pixels[idx + 1] = (out_g * 255.0).clamp(0.0, 255.0).round() as u8;
                        pixels[idx + 2] = (out_b * 255.0).clamp(0.0, 255.0).round() as u8;
                        pixels[idx + 3] = (out_a * 255.0).clamp(0.0, 255.0).round() as u8;
                    }
                }
            }
        }
    }

    /// Interpola pontos continuamente entre (x0, y0) e (x1, y1) usando o espaçamento configurado.
    pub fn stroke_line(
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
        settings: &BrushSettings,
        color: [u8; 4],
        is_eraser: bool,
        pixels: &mut [u8],
        width: usize,
        height: usize,
        selection: Option<&DocumentSelection>,
    ) {
        let dx = x1 - x0;
        let dy = y1 - y0;
        let distance = (dx * dx + dy * dy).sqrt();

        let step_size = (settings.size * settings.spacing).max(1.0);
        let steps = (distance / step_size).ceil() as usize;

        if steps == 0 {
            Self::render_dab(x0, y0, settings, color, is_eraser, pixels, width, height, selection);
            return;
        }

        for i in 0..=steps {
            let t = i as f64 / steps as f64;
            let px = x0 + dx * t;
            let py = y0 + dy * t;
            Self::render_dab(px, py, settings, color, is_eraser, pixels, width, height, selection);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brush_paint_and_erase() {
        let width = 50;
        let height = 50;
        let mut pixels = vec![0u8; width * height * 4];

        let mut settings = BrushSettings::default();
        settings.size = 10.0;
        settings.hardness = 1.0;
        settings.opacity = 1.0;
        settings.flow = 1.0;

        let red = [255, 0, 0, 255];

        // Pintar ponto no centro (25, 25)
        BrushEngine::render_dab(25.0, 25.0, &settings, red, false, &mut pixels, width, height, None);

        let center_idx = (25 * width + 25) * 4;
        assert_eq!(pixels[center_idx], 255); // R
        assert_eq!(pixels[center_idx + 1], 0);   // G
        assert_eq!(pixels[center_idx + 2], 0);   // B
        assert_eq!(pixels[center_idx + 3], 255); // A

        // Apagar ponto no centro com a borracha
        BrushEngine::render_dab(25.0, 25.0, &settings, red, true, &mut pixels, width, height, None);
        assert_eq!(pixels[center_idx + 3], 0); // Alfa apagado para 0
    }
}
