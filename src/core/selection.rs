use crate::core::session::SelectionMode;
use crate::ffi::c_bindings::wand_mask;

#[derive(Debug, Clone, PartialEq)]
pub struct DocumentSelection {
    pub width: usize,
    pub height: usize,
    /// Máscara de seleção em tons de cinza: 0 (não selecionado) a 255 (totalmente selecionado)
    pub mask: Vec<u8>,
}

impl DocumentSelection {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            mask: vec![0; width * height],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.mask.iter().all(|&v| v == 0)
    }

    pub fn clear(&mut self) {
        self.mask.fill(0);
    }

    pub fn select_all(&mut self) {
        self.mask.fill(255);
    }

    pub fn invert(&mut self) {
        for v in &mut self.mask {
            *v = 255 - *v;
        }
    }

    pub fn contains(&self, x: usize, y: usize) -> bool {
        if x < self.width && y < self.height {
            self.mask[y * self.width + x] > 0
        } else {
            false
        }
    }

    pub fn get_value(&self, x: usize, y: usize) -> u8 {
        if x < self.width && y < self.height {
            self.mask[y * self.width + x]
        } else {
            0
        }
    }

    /// Seleção Retangular (Marquee Tool)
    pub fn select_rect(&mut self, x: usize, y: usize, w: usize, h: usize, mode: SelectionMode) {
        let x_end = (x + w).min(self.width);
        let y_end = (y + h).min(self.height);

        match mode {
            SelectionMode::Replace => {
                self.clear();
                for row in y..y_end {
                    let start = row * self.width + x;
                    let end = row * self.width + x_end;
                    self.mask[start..end].fill(255);
                }
            }
            SelectionMode::Add => {
                for row in y..y_end {
                    for col in x..x_end {
                        self.mask[row * self.width + col] = 255;
                    }
                }
            }
            SelectionMode::Subtract => {
                for row in y..y_end {
                    for col in x..x_end {
                        self.mask[row * self.width + col] = 0;
                    }
                }
            }
            SelectionMode::Intersect => {
                for row in 0..self.height {
                    for col in 0..self.width {
                        let inside = col >= x && col < x_end && row >= y && row < y_end;
                        if !inside {
                            self.mask[row * self.width + col] = 0;
                        }
                    }
                }
            }
        }
    }

    /// Seleção Elíptica
    pub fn select_ellipse(&mut self, cx: f64, cy: f64, rx: f64, ry: f64, mode: SelectionMode) {
        if rx <= 0.0 || ry <= 0.0 {
            return;
        }
        let rx2 = rx * rx;
        let ry2 = ry * ry;

        let mut new_mask = vec![0u8; self.width * self.height];
        let min_x = ((cx - rx).max(0.0)) as usize;
        let max_x = ((cx + rx).min((self.width - 1) as f64)) as usize;
        let min_y = ((cy - ry).max(0.0)) as usize;
        let max_y = ((cy + ry).min((self.height - 1) as f64)) as usize;

        for y in min_y..=max_y {
            let dy = y as f64 - cy;
            let dy2 = dy * dy;
            for x in min_x..=max_x {
                let dx = x as f64 - cx;
                let dx2 = dx * dx;
                if (dx2 / rx2) + (dy2 / ry2) <= 1.0 {
                    new_mask[y * self.width + x] = 255;
                }
            }
        }

        self.apply_mode(&new_mask, mode);
    }

    /// Seleção por Varinha Mágica (Magic Wand Tool via c_wand_mask)
    pub fn select_wand(
        &mut self,
        image_pixels: &[u8],
        start_x: usize,
        start_y: usize,
        tolerance: f64,
        contiguous: bool,
        mode: SelectionMode,
    ) {
        let mut new_mask = vec![0u8; self.width * self.height];
        let stride = self.width * 4;

        unsafe {
            wand_mask(
                image_pixels.as_ptr(),
                self.width,
                self.height,
                stride,
                start_x,
                start_y,
                0,
                tolerance as i32,
                if contiguous { 1 } else { 0 },
                new_mask.as_mut_ptr(),
            );
        }

        self.apply_mode(&new_mask, mode);
    }

    fn apply_mode(&mut self, other: &[u8], mode: SelectionMode) {
        match mode {
            SelectionMode::Replace => {
                self.mask.copy_from_slice(other);
            }
            SelectionMode::Add => {
                for (dst, &src) in self.mask.iter_mut().zip(other.iter()) {
                    *dst = (*dst).max(src);
                }
            }
            SelectionMode::Subtract => {
                for (dst, &src) in self.mask.iter_mut().zip(other.iter()) {
                    if src > 0 {
                        *dst = 0;
                    }
                }
            }
            SelectionMode::Intersect => {
                for (dst, &src) in self.mask.iter_mut().zip(other.iter()) {
                    if src == 0 {
                        *dst = 0;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_rect_modes() {
        let mut sel = DocumentSelection::new(100, 100);
        assert!(sel.is_empty());

        // Replace
        sel.select_rect(10, 10, 20, 20, SelectionMode::Replace);
        assert!(!sel.is_empty());
        assert!(sel.contains(15, 15));
        assert!(!sel.contains(5, 5));

        // Add
        sel.select_rect(50, 50, 10, 10, SelectionMode::Add);
        assert!(sel.contains(15, 15));
        assert!(sel.contains(55, 55));

        // Subtract
        sel.select_rect(10, 10, 20, 20, SelectionMode::Subtract);
        assert!(!sel.contains(15, 15));
        assert!(sel.contains(55, 55));

        // Invert
        sel.invert();
        assert!(sel.contains(15, 15));
        assert!(!sel.contains(55, 55));
    }
}
