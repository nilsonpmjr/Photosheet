//! Pool e Cache de Texturas Vulkan para Camadas e Viewport.
//! Traduzido de Compositor/Rendering/DownsampleCache.swift e GPUCanvas.swift.

use crate::render::vulkan_context::VulkanContext;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use wgpu::{
    Device, Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, Queue, Texture, TextureAspect,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor,
};

pub struct TextureCache {
    device: Arc<Device>,
    queue: Arc<Queue>,
    textures: HashMap<Uuid, Texture>,
}

impl TextureCache {
    pub fn new(context: &VulkanContext) -> Self {
        Self {
            device: Arc::clone(&context.device),
            queue: Arc::clone(&context.queue),
            textures: HashMap::new(),
        }
    }

    /// Cria ou reutiliza uma textura RGBA8 com as dimensões fornecidas.
    pub fn get_or_create(&mut self, id: Uuid, width: u32, height: u32) -> &Texture {
        self.textures.entry(id).or_insert_with(|| {
            self.device.create_texture(&TextureDescriptor {
                label: Some(&format!("Layer Texture {}", id)),
                size: Extent3d {
                    width: width.max(1),
                    height: height.max(1),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8Unorm,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::COPY_SRC
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
        })
    }

    /// Faz upload de um buffer de pixels RGBA da CPU para a textura GPU.
    pub fn upload_pixels(&mut self, id: Uuid, width: u32, height: u32, pixels: &[u8]) {
        self.get_or_create(id, width, height);
        let texture = self.textures.get(&id).expect("Texture must exist");
        self.queue.write_texture(
            ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            pixels,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
        );
    }

    pub fn get_view(&mut self, id: Uuid, width: u32, height: u32) -> TextureView {
        let texture = self.get_or_create(id, width, height);
        texture.create_view(&TextureViewDescriptor::default())
    }

    pub fn remove(&mut self, id: &Uuid) {
        self.textures.remove(id);
    }

    pub fn clear(&mut self) {
        self.textures.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_cache_upload_and_view() {
        if let Ok(ctx) = VulkanContext::new() {
            let mut cache = TextureCache::new(&ctx);
            let id = Uuid::new_v4();
            let pixels = vec![255u8; 16 * 16 * 4];
            cache.upload_pixels(id, 16, 16, &pixels);
            let _view = cache.get_view(id, 16, 16);
            assert!(cache.textures.contains_key(&id));
            cache.remove(&id);
            assert!(!cache.textures.contains_key(&id));
        }
    }
}
