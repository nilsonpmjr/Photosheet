//! Inicialização e Gerenciamento do Contexto Gráfico Vulkan via wgpu.
//! Compatível com GPUs AMD (Mesa RADV), NVIDIA (Driver Oficial) e CPU fallback (Lavapipe).

use std::sync::Arc;
use wgpu::{
    Backends, Device, DeviceDescriptor, Features, Instance, InstanceDescriptor, Limits,
    PowerPreference, Queue, RequestAdapterOptions,
};

#[derive(Clone)]
pub struct VulkanContext {
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub adapter_info: wgpu::AdapterInfo,
}

impl VulkanContext {
    /// Inicializa o contexto gráfico Vulkan com suporte assíncrono/síncrono.
    pub fn new() -> Result<Self, String> {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::VULKAN | Backends::GL,
            ..Default::default()
        });

        let adapter = pollster::block_on(async {
            instance
                .request_adapter(&RequestAdapterOptions {
                    power_preference: PowerPreference::HighPerformance,
                    compatible_surface: None,
                    force_fallback_adapter: false,
                })
                .await
        });

        let adapter = match adapter {
            Some(a) => a,
            None => {
                // Fallback para adaptadores de software (Lavapipe/CPU) se nenhuma GPU for encontrada
                pollster::block_on(async {
                    instance
                        .request_adapter(&RequestAdapterOptions {
                            power_preference: PowerPreference::None,
                            compatible_surface: None,
                            force_fallback_adapter: true,
                        })
                        .await
                })
                .ok_or_else(|| {
                    "Nenhum adaptador Vulkan ou OpenGL compatível encontrado no Linux.".to_string()
                })?
            }
        };

        let adapter_info = adapter.get_info();

        let (device, queue) = pollster::block_on(async {
            adapter
                .request_device(
                    &DeviceDescriptor {
                        label: Some("Photosheet Vulkan Device"),
                        required_features: Features::empty(),
                        required_limits: Limits::default(),
                        memory_hints: Default::default(),
                    },
                    None,
                )
                .await
        })
        .map_err(|e| format!("Falha ao criar dispositivo Vulkan: {:?}", e))?;

        Ok(Self {
            instance: Arc::new(instance),
            device: Arc::new(device),
            queue: Arc::new(queue),
            adapter_info,
        })
    }

    pub fn device_name(&self) -> &str {
        &self.adapter_info.name
    }

    pub fn driver_info(&self) -> &str {
        &self.adapter_info.driver_info
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vulkan_context_initialization() {
        let ctx = VulkanContext::new();
        assert!(
            ctx.is_ok(),
            "Contexto Vulkan deve inicializar no Linux: {:?}",
            ctx.err()
        );
        let ctx = ctx.unwrap();
        println!(
            "GPU detectada: {} ({})",
            ctx.device_name(),
            ctx.driver_info()
        );
    }
}
