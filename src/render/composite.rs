//! Pipeline de Renderização e Composição de Camadas em GPU.
//! Traduzido de Compositor/Rendering/LayerRenderer.swift e GPUCanvas.swift.

use crate::render::vulkan_context::VulkanContext;
use bytemuck::{Pod, Zeroable};
use std::sync::Arc;
use wgpu::{
    include_wgsl, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
    Buffer, BufferBindingType, BufferDescriptor, BufferUsages, ColorTargetState,
    ColorWrites, Device, FragmentState, MultisampleState, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, Sampler,
    SamplerBindingType, SamplerDescriptor, ShaderStages, TextureFormat, TextureSampleType,
    TextureView, TextureViewDimension, VertexState,
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct CompositeUniforms {
    pub opacity: f32,
    pub folder_opacity: f32,
    pub blend_mode: u32,
    pub has_mask: u32,
}

pub struct LayerCompositePipeline {
    pub pipeline: RenderPipeline,
    pub bind_group_layout: BindGroupLayout,
    pub sampler: Sampler,
    pub uniform_buffer: Buffer,
    device: Arc<Device>,
    queue: Arc<Queue>,
}

impl LayerCompositePipeline {
    pub fn new(context: &VulkanContext, target_format: TextureFormat) -> Self {
        let device = &context.device;
        let queue = &context.queue;

        let shader = device.create_shader_module(include_wgsl!("shaders/composite.wgsl"));

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Composite Uniform Buffer"),
            size: std::mem::size_of::<CompositeUniforms>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Composite Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Composite Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Composite Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Layer Composite Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: target_format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            bind_group_layout,
            sampler,
            uniform_buffer,
            device: Arc::clone(device),
            queue: Arc::clone(queue),
        }
    }

    pub fn render(
        &self,
        layer_view: &TextureView,
        backdrop_view: &TextureView,
        mask_view: &TextureView,
        output_view: &TextureView,
        uniforms: CompositeUniforms,
    ) {
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Composite Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&self.sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(layer_view),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(backdrop_view),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::TextureView(mask_view),
                },
            ],
        });

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Composite Command Encoder"),
        });

        {
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Composite Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &bind_group, &[]);
            rpass.draw(0..3, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composite_pipeline_creation_and_render() {
        if let Ok(ctx) = VulkanContext::new() {
            let pipeline = LayerCompositePipeline::new(&ctx, TextureFormat::Rgba8Unorm);
            let mut cache = crate::render::texture_cache::TextureCache::new(&ctx);
            let layer_id = uuid::Uuid::new_v4();
            let backdrop_id = uuid::Uuid::new_v4();
            let mask_id = uuid::Uuid::new_v4();
            let output_id = uuid::Uuid::new_v4();

            let red_pixel = [255, 0, 0, 255];
            let blue_pixel = [0, 0, 255, 255];
            let white_pixel = [255, 255, 255, 255];

            cache.upload_pixels(layer_id, 1, 1, &red_pixel);
            cache.upload_pixels(backdrop_id, 1, 1, &blue_pixel);
            cache.upload_pixels(mask_id, 1, 1, &white_pixel);

            let layer_view = cache.get_view(layer_id, 1, 1);
            let backdrop_view = cache.get_view(backdrop_id, 1, 1);
            let mask_view = cache.get_view(mask_id, 1, 1);
            let output_view = cache.get_view(output_id, 1, 1);

            pipeline.render(
                &layer_view,
                &backdrop_view,
                &mask_view,
                &output_view,
                CompositeUniforms {
                    opacity: 1.0,
                    folder_opacity: 1.0,
                    blend_mode: 0,
                    has_mask: 1,
                },
            );
        }
    }
}

