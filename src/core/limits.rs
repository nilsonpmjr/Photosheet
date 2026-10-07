//! Ceilings de tamanho e limites de memória para documentos.
//! Traduzido de Compositor/Document/DocumentLimits.swift.

pub struct DocumentLimits;

impl DocumentLimits {
    /// Maior lado suportado, em pixels, de qualquer tela, camada, máscara ou superfície gerada (30.000 px).
    pub const MAX_SIDE: usize = 30_000;

    /// Maior superfície única permitida (canvas, exportação, alvo de filtro): 200 megapixels.
    pub const MAX_SURFACE_PIXELS: usize = 200_000_000;

    /// Orçamento total de pixels de raster importados por documento:
    /// no Linux, dimensionado a partir da memória física (mínimo 200 MP, máximo 800 MP).
    pub fn document_pixel_budget() -> usize {
        let total_mem = if let Ok(mem) = sys_info_total_memory() {
            mem
        } else {
            16 * 1024 * 1024 * 1024 // Fallback 16 GB
        };
        let quarter = total_mem / 16; // em bytes dividido por 4 bytes/pixel
        let min_b = Self::MAX_SURFACE_PIXELS;
        let max_b = 800_000_000;
        quarter.clamp(min_b as u64, max_b as u64) as usize
    }

    pub fn max_surface_megapixels() -> usize {
        Self::MAX_SURFACE_PIXELS / 1_000_000
    }

    pub fn document_budget_megapixels() -> usize {
        Self::document_pixel_budget() / 1_000_000
    }
}

fn sys_info_total_memory() -> Result<u64, ()> {
    // Lê a memória física de /proc/meminfo no Linux
    if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<u64>() {
                        return Ok(kb * 1024);
                    }
                }
            }
        }
    }
    Err(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limits() {
        assert_eq!(DocumentLimits::MAX_SIDE, 30_000);
        assert_eq!(DocumentLimits::MAX_SURFACE_PIXELS, 200_000_000);
        assert!(DocumentLimits::document_pixel_budget() >= DocumentLimits::MAX_SURFACE_PIXELS);
    }
}
