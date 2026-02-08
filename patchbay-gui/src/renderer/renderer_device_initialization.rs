//! Renderer device initialization helpers.

use std::sync::Arc;

use crate::host::GuiError;
use crate::logging::log_line_safe;

use super::RendererDevice;

impl RendererDevice {
    /// Create a new device and queue without binding to a specific surface.
    pub fn new() -> Result<Self, GuiError> {
        log_line_safe("renderer_device: create begin");
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::from_env().unwrap_or(wgpu::Backends::PRIMARY),
            ..Default::default()
        });
        log_line_safe("renderer_device: instance created");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .map_err(|err| {
            log_line_safe(&format!("renderer_device: request_adapter error: {err:?}"));
            GuiError::AdapterNotFound
        })?;
        log_line_safe("renderer_device: adapter acquired");

        let required_features =
            adapter.features() & (wgpu::Features::CLEAR_TEXTURE | wgpu::Features::PIPELINE_CACHE);
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("patchbay-gui-device"),
            required_features,
            required_limits: wgpu::Limits::default(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
        }))
        .map_err(|err| {
            log_line_safe(&format!("renderer_device: request_device error: {err:?}"));
            GuiError::Device(err)
        })?;
        log_line_safe("renderer_device: device created");
        device.on_uncaptured_error(Arc::new(|error| {
            log_line_safe(&format!("renderer_device: uncaptured wgpu error: {error}"));
        }));

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
        })
    }
}
