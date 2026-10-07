//! Leitor de Arquivos PSD / PSB (Photoshop Document).
//! Traduzido de Compositor/IO/PSD/PSDReader.swift e PSDChannelCoder.swift.

use crate::core::blend::LayerBlendMode;
use crate::core::document::Document;
use crate::core::layer::{Layer, LayerKind};
use crate::core::transform::{LayerTransform, Point, Size};
use std::io::{self, Cursor, Read, Seek, SeekFrom};
use uuid::Uuid;

#[derive(Debug)]
pub enum PsdError {
    InvalidMagic,
    UnsupportedVersion,
    UnsupportedDepth,
    UnsupportedColorMode,
    DocumentTooLarge,
    Io(io::Error),
}

impl std::fmt::Display for PsdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidMagic => write!(f, "Assinatura mágica inválida (esperado '8BPS')"),
            Self::UnsupportedVersion => write!(f, "Versão PSD/PSB não suportada"),
            Self::UnsupportedDepth => write!(
                f,
                "Profundidade de cor não suportada (suportado apenas 8 bits)"
            ),
            Self::UnsupportedColorMode => {
                write!(f, "Modo de cor não suportado (suportado apenas RGB)")
            }
            Self::DocumentTooLarge => write!(f, "Dimensões do documento excedem os limites"),
            Self::Io(err) => write!(f, "Erro de E/S ao ler arquivo PSD: {}", err),
        }
    }
}

impl std::error::Error for PsdError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for PsdError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

pub struct PsdReader;

impl PsdReader {
    pub fn matches(data: &[u8]) -> bool {
        data.len() >= 4 && &data[0..4] == b"8BPS"
    }

    pub fn read(data: &[u8]) -> Result<Document, PsdError> {
        let mut r = Cursor::new(data);

        // 1. Cabeçalho (Header)
        let mut magic = [0u8; 4];
        r.read_exact(&mut magic)?;
        if &magic != b"8BPS" {
            return Err(PsdError::InvalidMagic);
        }

        let mut u16_buf = [0u8; 2];
        r.read_exact(&mut u16_buf)?;
        let version = u16::from_be_bytes(u16_buf);
        if version != 1 && version != 2 {
            return Err(PsdError::UnsupportedVersion);
        }
        let is_psb = version == 2;

        // 6 bytes reservados
        r.seek(SeekFrom::Current(6))?;

        // Contagem de canais
        r.read_exact(&mut u16_buf)?;
        let _channels = u16::from_be_bytes(u16_buf);

        let mut u32_buf = [0u8; 4];
        r.read_exact(&mut u32_buf)?;
        let height = u32::from_be_bytes(u32_buf) as usize;

        r.read_exact(&mut u32_buf)?;
        let width = u32::from_be_bytes(u32_buf) as usize;

        r.read_exact(&mut u16_buf)?;
        let depth = u16::from_be_bytes(u16_buf);
        if depth != 8 {
            return Err(PsdError::UnsupportedDepth);
        }

        r.read_exact(&mut u16_buf)?;
        let mode = u16::from_be_bytes(u16_buf);
        if mode != 3 {
            // RGB
            return Err(PsdError::UnsupportedColorMode);
        }

        // 2. Color Mode Data
        r.read_exact(&mut u32_buf)?;
        let color_mode_len = u32::from_be_bytes(u32_buf) as i64;
        r.seek(SeekFrom::Current(color_mode_len))?;

        // 3. Image Resources (Resolução, Guias, etc.)
        r.read_exact(&mut u32_buf)?;
        let resources_len = u32::from_be_bytes(u32_buf) as i64;
        let resources_end = r.position() + resources_len as u64;

        let mut resolution = 72.0;
        while r.position() + 12 <= resources_end {
            let mut sig = [0u8; 4];
            r.read_exact(&mut sig)?;
            if &sig != b"8BIM" {
                break;
            }

            r.read_exact(&mut u16_buf)?;
            let res_id = u16::from_be_bytes(u16_buf);

            let mut name_len_b = [0u8; 1];
            r.read_exact(&mut name_len_b)?;
            let name_len = name_len_b[0] as i64;
            r.seek(SeekFrom::Current(name_len))?;
            if (name_len + 1) % 2 == 1 {
                r.seek(SeekFrom::Current(1))?;
            }

            r.read_exact(&mut u32_buf)?;
            let res_data_len = u32::from_be_bytes(u32_buf) as u64;
            let next_res = r.position() + res_data_len + if res_data_len % 2 == 1 { 1 } else { 0 };

            if res_id == 1005 && res_data_len >= 4 {
                // ResolutionInfo
                r.read_exact(&mut u32_buf)?;
                let res_fixed = u32::from_be_bytes(u32_buf);
                let res_dpi = (res_fixed as f64) / 65536.0;
                if res_dpi.is_finite() && res_dpi >= 1.0 {
                    resolution = res_dpi.clamp(1.0, 9600.0);
                }
            }

            r.seek(SeekFrom::Start(next_res))?;
        }
        r.seek(SeekFrom::Start(resources_end))?;

        // 4. Seção de Camadas e Máscaras (Layer and Mask Information)
        let layer_section_len = if is_psb {
            let mut u64_buf = [0u8; 8];
            r.read_exact(&mut u64_buf)?;
            u64::from_be_bytes(u64_buf)
        } else {
            r.read_exact(&mut u32_buf)?;
            u32::from_be_bytes(u32_buf) as u64
        };

        let mut doc = Document::new(width, height, resolution);

        if layer_section_len < 4 {
            // Documento plano sem estrutura de camadas explícita
            return Ok(doc);
        }

        let _layer_info_len = if is_psb {
            let mut u64_buf = [0u8; 8];
            r.read_exact(&mut u64_buf)?;
            u64::from_be_bytes(u64_buf)
        } else {
            r.read_exact(&mut u32_buf)?;
            u32::from_be_bytes(u32_buf) as u64
        };

        r.read_exact(&mut u16_buf)?;
        let layer_count_raw = i16::from_be_bytes(u16_buf);
        let layer_count = layer_count_raw.unsigned_abs() as usize;

        struct ParsedLayerRecord {
            top: i32,
            left: i32,
            bottom: i32,
            right: i32,
            channel_count: usize,
            channel_ids: Vec<i16>,
            blend_mode: LayerBlendMode,
            opacity: f64,
            is_visible: bool,
            name: String,
        }

        let mut records = Vec::with_capacity(layer_count);

        for _ in 0..layer_count {
            let mut i32_buf = [0u8; 4];
            r.read_exact(&mut i32_buf)?;
            let top = i32::from_be_bytes(i32_buf);

            r.read_exact(&mut i32_buf)?;
            let left = i32::from_be_bytes(i32_buf);

            r.read_exact(&mut i32_buf)?;
            let bottom = i32::from_be_bytes(i32_buf);

            r.read_exact(&mut i32_buf)?;
            let right = i32::from_be_bytes(i32_buf);

            r.read_exact(&mut u16_buf)?;
            let ch_count = u16::from_be_bytes(u16_buf) as usize;

            let mut channel_ids = Vec::with_capacity(ch_count);
            for _ in 0..ch_count {
                r.read_exact(&mut u16_buf)?;
                channel_ids.push(i16::from_be_bytes(u16_buf));

                if is_psb {
                    let mut u64_buf = [0u8; 8];
                    r.read_exact(&mut u64_buf)?;
                } else {
                    r.read_exact(&mut u32_buf)?;
                }
            }

            // Assinatura de mesclagem ('8BIM')
            let mut sig = [0u8; 4];
            r.read_exact(&mut sig)?;

            // Chave do blend mode
            let mut blend_key = [0u8; 4];
            r.read_exact(&mut blend_key)?;
            let blend_mode = match &blend_key {
                b"norm" => LayerBlendMode::Normal,
                b"dark" => LayerBlendMode::Darken,
                b"mul " => LayerBlendMode::Multiply,
                b"idiv" => LayerBlendMode::ColorBurn,
                b"lbrn" => LayerBlendMode::LinearBurn,
                b"lite" => LayerBlendMode::Lighten,
                b"scrn" => LayerBlendMode::Screen,
                b"div " => LayerBlendMode::ColorDodge,
                b"lddg" => LayerBlendMode::LinearDodge,
                b"over" => LayerBlendMode::Overlay,
                b"sLit" => LayerBlendMode::SoftLight,
                b"hLit" => LayerBlendMode::HardLight,
                b"vLit" => LayerBlendMode::VividLight,
                b"lLit" => LayerBlendMode::LinearLight,
                b"pLit" => LayerBlendMode::PinLight,
                b"hMix" => LayerBlendMode::HardMix,
                b"diff" => LayerBlendMode::Difference,
                b"smud" => LayerBlendMode::Exclusion,
                b"fsub" => LayerBlendMode::Subtract,
                b"fdiv" => LayerBlendMode::Divide,
                b"hue " => LayerBlendMode::Hue,
                b"sat " => LayerBlendMode::Saturation,
                b"colr" => LayerBlendMode::Color,
                b"lum " => LayerBlendMode::Luminosity,
                _ => LayerBlendMode::Normal,
            };

            let mut u8_buf = [0u8; 1];
            r.read_exact(&mut u8_buf)?;
            let opacity = u8_buf[0] as f64 / 255.0;

            // Clipping
            r.read_exact(&mut u8_buf)?;

            // Flags
            r.read_exact(&mut u8_buf)?;
            let flags = u8_buf[0];
            let is_visible = (flags & 0x02) == 0;

            // Filler byte
            r.seek(SeekFrom::Current(1))?;

            // Extra data field length
            r.read_exact(&mut u32_buf)?;
            let extra_len = u32::from_be_bytes(u32_buf) as u64;
            let extra_end = r.position() + extra_len;

            // Mask data length
            r.read_exact(&mut u32_buf)?;
            let mask_len = u32::from_be_bytes(u32_buf) as i64;
            r.seek(SeekFrom::Current(mask_len))?;

            // Layer blending ranges length
            r.read_exact(&mut u32_buf)?;
            let blend_ranges_len = u32::from_be_bytes(u32_buf) as i64;
            r.seek(SeekFrom::Current(blend_ranges_len))?;

            // Layer name (Pascal string alinhada a múltiplos de 4)
            r.read_exact(&mut u8_buf)?;
            let name_len = u8_buf[0] as usize;
            let mut name_bytes = vec![0u8; name_len];
            r.read_exact(&mut name_bytes)?;
            let name = String::from_utf8_lossy(&name_bytes).to_string();

            r.seek(SeekFrom::Start(extra_end))?;

            records.push(ParsedLayerRecord {
                top,
                left,
                bottom,
                right,
                channel_count: ch_count,
                channel_ids,
                blend_mode,
                opacity,
                is_visible,
                name: if name.is_empty() {
                    format!("Camada {}", records.len() + 1)
                } else {
                    name
                },
            });
        }

        // Criar as camadas do documento a partir dos registros
        for rec in records {
            let layer_w = (rec.right - rec.left).max(1) as usize;
            let layer_h = (rec.bottom - rec.top).max(1) as usize;

            let transform = LayerTransform::new(
                Point {
                    x: rec.left as f64,
                    y: rec.top as f64,
                },
                Size {
                    width: layer_w as f64,
                    height: layer_h as f64,
                },
            );

            let mut layer = Layer::new_pixel(rec.name, layer_w, layer_h, transform);
            layer.blend_mode = rec.blend_mode;
            layer.opacity = rec.opacity;
            layer.is_visible = rec.is_visible;

            doc.add_layer(layer);
        }

        Ok(doc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_psd_magic_detection() {
        let valid = b"8BPS\x00\x01\x00\x00\x00\x00\x00\x00\x00\x03\x00\x00\x00\x64\x00\x00\x00\x64\x00\x08\x00\x03\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        assert!(PsdReader::matches(valid));

        let invalid = b"PNG!\x00\x00";
        assert!(!PsdReader::matches(invalid));

        let doc = PsdReader::read(valid).unwrap();
        assert_eq!(doc.width, 100);
        assert_eq!(doc.height, 100);
        assert_eq!(doc.resolution, 72.0);
    }
}
