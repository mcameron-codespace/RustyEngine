use std::sync::Arc;
use winit::window::Window;

/// The graphics context housing all core `wgpu` structures.
/// This layer coordinates the hardware adapter, device queues, and screen swapchain.
pub struct RenderContext<'window> {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'window>,
    pub config: wgpu::SurfaceConfiguration,
}

impl<'window> RenderContext<'window> {
    /// Create a new graphics context, prioritizing Vulkan but falling back to OpenGL/GLES if necessary.
    pub fn new(window: Arc<Window>) -> Self {
        // Initialize instance favoring Vulkan and GL backends
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN | wgpu::Backends::GL,
            dx12_shader_compiler: Default::default(),
            flags: wgpu::InstanceFlags::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::default(),
        });

        // Create the presentation surface linked to the OS window context
        let surface = instance.create_surface(window.clone()).expect("Failed to create window surface");

        // Request an adapter, favoring high-performance discrete GPUs
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("Failed to acquire suitable graphics adapter");

        // Output GPU info to the terminal to verify active backend (Phase 2.2 Verification)
        let info = adapter.get_info();
        println!("Aether Graphics initialized on: {} [{:?}]", info.name, info.backend);

        // Request a connection to the physical device
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Aether Main Device"),
                required_features: wgpu::Features::empty() 
                    | wgpu::Features::MULTI_DRAW_INDIRECT 
                    | wgpu::Features::INDIRECT_FIRST_INSTANCE,
                required_limits: wgpu::Limits::default(),
            },
            None,
        ))
        .expect("Failed to create logical rendering device");

        // Configure the initial swapchain presentation limits
        let size = window.inner_size();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo, // Equivalent to VSync
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        Self {
            instance,
            adapter,
            device,
            queue,
            surface,
            config,
        }
    }

    /// Reconfigure the presentation surface when the OS window is resized.
    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        if new_width > 0 && new_height > 0 {
            self.config.width = new_width;
            self.config.height = new_height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}