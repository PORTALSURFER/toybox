//! Renderer device initialization helpers.

use std::sync::Arc;

use crate::host::GuiError;
use crate::logging::log_line_safe;

use super::RendererDevice;

const BASIC_RENDER_DRIVER_VENDOR_ID: u32 = 5140;
const BASIC_RENDER_DRIVER_DEVICE_ID: u32 = 140;

/// The WGPU 29 DX12 software adapter crashes with `STATUS_ACCESS_VIOLATION`
/// while polling this renderer's work on the Windows runner. Keep this
/// identity match exact so other adapters still execute the frame-capture
/// assertion.
fn is_unsupported_frame_capture_adapter(info: &wgpu::AdapterInfo) -> bool {
    info.name == "Microsoft Basic Render Driver"
        && info.vendor == BASIC_RENDER_DRIVER_VENDOR_ID
        && info.device == BASIC_RENDER_DRIVER_DEVICE_ID
        && info.device_type == wgpu::DeviceType::Cpu
        && info.backend == wgpu::Backend::Dx12
}

impl RendererDevice {
    /// Create a new device and queue without binding to a specific surface.
    pub fn new() -> Result<Self, GuiError> {
        let backends = wgpu::Backends::from_env().unwrap_or(wgpu::Backends::PRIMARY);
        Self::new_with_backends(backends)
    }

    /// Create the device used by the Windows frame-capture regression.
    #[cfg(all(test, feature = "frame-capture", target_os = "windows"))]
    pub(crate) fn new_for_frame_capture() -> Result<Self, GuiError> {
        Self::new_with_backends_and_policy(wgpu::Backends::DX12, true)
    }

    /// Initialize a device using the supplied WGPU backend set.
    fn new_with_backends(backends: wgpu::Backends) -> Result<Self, GuiError> {
        Self::new_with_backends_and_policy(backends, false)
    }

    fn new_with_backends_and_policy(
        backends: wgpu::Backends,
        reject_basic_render_driver: bool,
    ) -> Result<Self, GuiError> {
        log_line_safe("renderer_device: create begin");
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            ..wgpu::InstanceDescriptor::new_without_display_handle()
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
        if reject_basic_render_driver && is_unsupported_frame_capture_adapter(&adapter.get_info()) {
            log_line_safe(
                "renderer_device: frame capture unavailable on Microsoft Basic Render Driver",
            );
            return Err(GuiError::FrameCaptureUnavailable);
        }
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

#[cfg(test)]
mod tests {
    use super::is_unsupported_frame_capture_adapter;

    fn basic_render_driver_info() -> wgpu::AdapterInfo {
        wgpu::AdapterInfo {
            name: "Microsoft Basic Render Driver".to_owned(),
            vendor: 5140,
            device: 140,
            device_type: wgpu::DeviceType::Cpu,
            device_pci_bus_id: String::new(),
            driver: "10.0.26100.33296".to_owned(),
            driver_info: String::new(),
            backend: wgpu::Backend::Dx12,
            subgroup_min_size: 4,
            subgroup_max_size: 4,
            transient_saves_memory: false,
        }
    }

    #[test]
    fn only_exact_basic_render_driver_is_unavailable_for_capture() {
        let mut info = basic_render_driver_info();
        assert!(is_unsupported_frame_capture_adapter(&info));

        info.name = "Microsoft Basic Render Driver (other)".to_owned();
        assert!(!is_unsupported_frame_capture_adapter(&info));

        info = basic_render_driver_info();
        info.vendor += 1;
        assert!(!is_unsupported_frame_capture_adapter(&info));

        info = basic_render_driver_info();
        info.device += 1;
        assert!(!is_unsupported_frame_capture_adapter(&info));

        info = basic_render_driver_info();
        info.device_type = wgpu::DeviceType::IntegratedGpu;
        assert!(!is_unsupported_frame_capture_adapter(&info));

        info = basic_render_driver_info();
        info.backend = wgpu::Backend::Vulkan;
        assert!(!is_unsupported_frame_capture_adapter(&info));
    }
}
