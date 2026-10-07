//! Pincel de Recuperação (Spot Healing Brush).
//! Traduzido de Compositor/Document/EditorSession+Brush.swift e HealPixels.c.

use crate::ffi::c_bindings::spot_heal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpotHealingMode {
    ContentAware = 0,
    ProximityMatch = 1,
    CreateTexture = 2,
}

pub struct HealEngine;

impl HealEngine {
    /// Aplica o algoritmo de recuperação pontual sobre a área mascarada de defeito (blemish mask).
    pub fn apply_spot_heal(
        rgba_pixels: &mut [u8],
        coverage_mask: &[u8],
        width: usize,
        height: usize,
        mode: SpotHealingMode,
        opacity: f32,
        seed: u32,
    ) -> bool {
        let stride = width * 4;
        let result = unsafe {
            spot_heal(
                rgba_pixels.as_mut_ptr(),
                coverage_mask.as_ptr(),
                width,
                height,
                stride,
                opacity,
                mode as i32,
                seed,
            )
        };
        result == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spot_heal_invocation() {
        let width = 32;
        let height = 32;
        let mut pixels = vec![200u8; width * height * 4];
        let mut mask = vec![0u8; width * height];

        // Marca um ponto preto no centro como imperfeição
        let center = 16 * width + 16;
        pixels[center * 4] = 0;
        pixels[center * 4 + 1] = 0;
        pixels[center * 4 + 2] = 0;
        mask[center] = 255;

        let ok = HealEngine::apply_spot_heal(
            &mut pixels,
            &mask,
            width,
            height,
            SpotHealingMode::ContentAware,
            1.0,
            42,
        );
        assert!(ok);
    }
}
