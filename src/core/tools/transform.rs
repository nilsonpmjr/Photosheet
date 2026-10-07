//! Manipulador de Transformação de Camada (Transform Gizmo).
//! Traduzido de Compositor/Document/TransformEdit.swift.

use crate::core::transform::{LayerTransform, Point, Size};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformHandle {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
    Body,
}

pub struct TransformGizmo;

impl TransformGizmo {
    pub const HANDLE_RADIUS: f64 = 8.0;

    /// Retorna as 8 posições das alças de controle nos cantos e centros das bordas.
    pub fn handle_positions(t: &LayerTransform) -> Vec<(TransformHandle, Point)> {
        let x = t.origin.x;
        let y = t.origin.y;
        let w = t.size.width;
        let h = t.size.height;

        vec![
            (TransformHandle::TopLeft, Point { x, y }),
            (TransformHandle::TopCenter, Point { x: x + w / 2.0, y }),
            (TransformHandle::TopRight, Point { x: x + w, y }),
            (TransformHandle::MiddleLeft, Point { x, y: y + h / 2.0 }),
            (TransformHandle::MiddleRight, Point { x: x + w, y: y + h / 2.0 }),
            (TransformHandle::BottomLeft, Point { x, y: y + h }),
            (TransformHandle::BottomCenter, Point { x: x + w / 2.0, y: y + h }),
            (TransformHandle::BottomRight, Point { x: x + w, y: y + h }),
        ]
    }

    /// Testa se um clique de mouse atingiu uma das alças de controle ou o corpo da caixa delimitadora.
    pub fn hit_test(t: &LayerTransform, p: Point) -> Option<TransformHandle> {
        let handles = Self::handle_positions(t);
        for (handle, pt) in handles {
            let dx = p.x - pt.x;
            let dy = p.y - pt.y;
            if (dx * dx + dy * dy).sqrt() <= Self::HANDLE_RADIUS {
                return Some(handle);
            }
        }

        // Testar se está dentro do corpo da camada para translação
        if p.x >= t.origin.x
            && p.x <= t.origin.x + t.size.width
            && p.y >= t.origin.y
            && p.y <= t.origin.y + t.size.height
        {
            Some(TransformHandle::Body)
        } else {
            None
        }
    }

    /// Aplica deslocamento (dx, dy) na alça selecionada atualizando a transformação.
    pub fn apply_drag(
        t: &mut LayerTransform,
        handle: TransformHandle,
        dx: f64,
        dy: f64,
        lock_aspect_ratio: bool,
    ) {
        match handle {
            TransformHandle::Body => {
                t.origin.x += dx;
                t.origin.y += dy;
            }
            TransformHandle::BottomRight => {
                let new_w = (t.size.width + dx).max(1.0);
                let mut new_h = (t.size.height + dy).max(1.0);
                if lock_aspect_ratio && t.size.width > 0.0 {
                    let ratio = t.size.height / t.size.width;
                    new_h = new_w * ratio;
                }
                t.size.width = new_w;
                t.size.height = new_h;
            }
            TransformHandle::BottomLeft => {
                let new_w = (t.size.width - dx).max(1.0);
                t.origin.x += t.size.width - new_w;
                t.size.width = new_w;
                t.size.height = (t.size.height + dy).max(1.0);
            }
            TransformHandle::TopRight => {
                t.size.width = (t.size.width + dx).max(1.0);
                let new_h = (t.size.height - dy).max(1.0);
                t.origin.y += t.size.height - new_h;
                t.size.height = new_h;
            }
            TransformHandle::TopLeft => {
                let new_w = (t.size.width - dx).max(1.0);
                let new_h = (t.size.height - dy).max(1.0);
                t.origin.x += t.size.width - new_w;
                t.origin.y += t.size.height - new_h;
                t.size.width = new_w;
                t.size.height = new_h;
            }
            TransformHandle::MiddleRight => {
                t.size.width = (t.size.width + dx).max(1.0);
            }
            TransformHandle::MiddleLeft => {
                let new_w = (t.size.width - dx).max(1.0);
                t.origin.x += t.size.width - new_w;
                t.size.width = new_w;
            }
            TransformHandle::BottomCenter => {
                t.size.height = (t.size.height + dy).max(1.0);
            }
            TransformHandle::TopCenter => {
                let new_h = (t.size.height - dy).max(1.0);
                t.origin.y += t.size.height - new_h;
                t.size.height = new_h;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gizmo_hit_test_and_drag() {
        let mut t = LayerTransform::new(Point { x: 100.0, y: 100.0 }, Size { width: 200.0, height: 100.0 });

        // Clica na alça TopLeft (100, 100)
        let hit = TransformGizmo::hit_test(&t, Point { x: 100.0, y: 100.0 });
        assert_eq!(hit, Some(TransformHandle::TopLeft));

        // Clica no centro da camada (200, 150)
        let hit_body = TransformGizmo::hit_test(&t, Point { x: 200.0, y: 150.0 });
        assert_eq!(hit_body, Some(TransformHandle::Body));

        // Arrastar o corpo
        TransformGizmo::apply_drag(&mut t, TransformHandle::Body, 10.0, 20.0, false);
        assert_eq!(t.origin.x, 110.0);
        assert_eq!(t.origin.y, 120.0);
    }
}
