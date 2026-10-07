//! Mecanismo de Renderização de Formas Geométricas (ShapeTool).
//! Traduzido de Compositor/Document/ShapeTool.swift.

use crate::core::session::{ShapeKind, ShapeToolSettings};

pub struct ShapeEngine;

impl ShapeEngine {
    /// Renderiza um retângulo, elipse ou linha com preenchimento e traçado no buffer RGBA.
    pub fn render_shape(
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        settings: &ShapeToolSettings,
        pixels: &mut [u8],
        buf_width: usize,
        buf_height: usize,
    ) {
        if w <= 0.0 || h <= 0.0 {
            return;
        }

        let min_x = (x.floor().max(0.0)) as usize;
        let max_x = ((x + w).ceil().min((buf_width - 1) as f64)) as usize;
        let min_y = (y.floor().max(0.0)) as usize;
        let max_y = ((y + h).ceil().min((buf_height - 1) as f64)) as usize;

        let stroke_w = settings.stroke_width;
        let has_stroke = stroke_w > 0.0 && settings.stroke_color[3] > 0;
        let has_fill = settings.fill_color[3] > 0;

        match settings.kind {
            ShapeKind::Rectangle => {
                let r = settings.corner_radius.min(w / 2.0).min(h / 2.0);

                for py in min_y..=max_y {
                    let fy = py as f64;
                    for px in min_x..=max_x {
                        let fx = px as f64;

                        let inside = if r > 0.0 {
                            let cx = if fx < x + r {
                                x + r
                            } else if fx > x + w - r {
                                x + w - r
                            } else {
                                fx
                            };
                            let cy = if fy < y + r {
                                y + r
                            } else if fy > y + h - r {
                                y + h - r
                            } else {
                                fy
                            };
                            let d2 = (fx - cx).powi(2) + (fy - cy).powi(2);
                            d2 <= r * r
                        } else {
                            fx >= x && fx <= x + w && fy >= y && fy <= y + h
                        };

                        if inside {
                            let is_border = has_stroke
                                && (fx - x < stroke_w
                                    || x + w - fx < stroke_w
                                    || fy - y < stroke_w
                                    || y + h - fy < stroke_w);

                            let col = if is_border {
                                settings.stroke_color
                            } else if has_fill {
                                settings.fill_color
                            } else {
                                continue;
                            };

                            let idx = (py * buf_width + px) * 4;
                            Self::blend_pixel(&mut pixels[idx..idx + 4], col);
                        }
                    }
                }
            }
            ShapeKind::Ellipse => {
                let cx = x + w / 2.0;
                let cy = y + h / 2.0;
                let rx = w / 2.0;
                let ry = h / 2.0;
                let rx2 = rx * rx;
                let ry2 = ry * ry;

                let inner_rx = (rx - stroke_w).max(0.0);
                let inner_ry = (ry - stroke_w).max(0.0);
                let inner_rx2 = inner_rx * inner_rx;
                let inner_ry2 = inner_ry * inner_ry;

                for py in min_y..=max_y {
                    let dy2 = (py as f64 - cy).powi(2);
                    for px in min_x..=max_x {
                        let dx2 = (px as f64 - cx).powi(2);
                        let outer_val = (dx2 / rx2) + (dy2 / ry2);

                        if outer_val <= 1.0 {
                            let is_stroke = has_stroke
                                && (inner_rx2 <= 0.0
                                    || (dx2 / inner_rx2) + (dy2 / inner_ry2) > 1.0);
                            let col = if is_stroke {
                                settings.stroke_color
                            } else if has_fill {
                                settings.fill_color
                            } else {
                                continue;
                            };

                            let idx = (py * buf_width + px) * 4;
                            Self::blend_pixel(&mut pixels[idx..idx + 4], col);
                        }
                    }
                }
            }
            ShapeKind::Line => {
                // Linha reta de (x, y) até (x + w, y + h)
                let x0 = x;
                let y0 = y;
                let x1 = x + w;
                let y1 = y + h;
                let dx = x1 - x0;
                let dy = y1 - y0;
                let len = (dx * dx + dy * dy).sqrt();

                if len > 0.0 {
                    let steps = len.ceil() as usize;
                    let radius = (stroke_w.max(1.0)) / 2.0;
                    for step in 0..=steps {
                        let t = step as f64 / steps as f64;
                        let lx = x0 + dx * t;
                        let ly = y0 + dy * t;
                        let l_min_x = ((lx - radius).floor().max(0.0)) as usize;
                        let l_max_x = ((lx + radius).ceil().min((buf_width - 1) as f64)) as usize;
                        let l_min_y = ((ly - radius).floor().max(0.0)) as usize;
                        let l_max_y = ((ly + radius).ceil().min((buf_height - 1) as f64)) as usize;

                        for py in l_min_y..=l_max_y {
                            for px in l_min_x..=l_max_x {
                                let d =
                                    ((px as f64 - lx).powi(2) + (py as f64 - ly).powi(2)).sqrt();
                                if d <= radius {
                                    let idx = (py * buf_width + px) * 4;
                                    Self::blend_pixel(
                                        &mut pixels[idx..idx + 4],
                                        settings.fill_color,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn blend_pixel(dst: &mut [u8], src: [u8; 4]) {
        let src_a = src[3] as f64 / 255.0;
        if src_a <= 0.0 {
            return;
        }

        let dst_a = dst[3] as f64 / 255.0;
        let out_a = src_a + dst_a * (1.0 - src_a);

        if out_a > 0.0 {
            let src_r = src[0] as f64 / 255.0;
            let src_g = src[1] as f64 / 255.0;
            let src_b = src[2] as f64 / 255.0;

            let dst_r = dst[0] as f64 / 255.0;
            let dst_g = dst[1] as f64 / 255.0;
            let dst_b = dst[2] as f64 / 255.0;

            let out_r = (src_r * src_a + dst_r * dst_a * (1.0 - src_a)) / out_a;
            let out_g = (src_g * src_a + dst_g * dst_a * (1.0 - src_a)) / out_a;
            let out_b = (src_b * src_a + dst_b * dst_a * (1.0 - src_a)) / out_a;

            dst[0] = (out_r * 255.0).clamp(0.0, 255.0).round() as u8;
            dst[1] = (out_g * 255.0).clamp(0.0, 255.0).round() as u8;
            dst[2] = (out_b * 255.0).clamp(0.0, 255.0).round() as u8;
            dst[3] = (out_a * 255.0).clamp(0.0, 255.0).round() as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_rect_render() {
        let width = 60;
        let height = 60;
        let mut pixels = vec![0u8; width * height * 4];

        let mut settings = ShapeToolSettings::default();
        settings.kind = ShapeKind::Rectangle;
        settings.fill_color = [0, 255, 0, 255]; // Verde
        settings.stroke_width = 0.0;

        ShapeEngine::render_shape(
            10.0,
            10.0,
            20.0,
            20.0,
            &settings,
            &mut pixels,
            width,
            height,
        );

        let inside_idx = (15 * width + 15) * 4;
        assert_eq!(pixels[inside_idx + 1], 255); // Verde

        let outside_idx = (5 * width + 5) * 4;
        assert_eq!(pixels[outside_idx + 1], 0);
    }
}
