use std::sync::Arc;

use winit::{dpi::PhysicalSize, window::Window};

use crate::block::block_on;

pub struct Ctx<'ctx> {
    surface: wgpu::Surface<'ctx>,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: PhysicalSize<u32>,
    // window: &'ctx Window,
}

impl<'ctx> Ctx<'ctx> {
    async fn async_new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let descriptor = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        };

        let instance = wgpu::Instance::new(&descriptor);

        let surface = instance
            .create_surface(Arc::clone(&window))
            .expect("Unable to create surface");

        let adapter_options = wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        };

        let adapter = instance.request_adapter(&adapter_options).await.unwrap();

        let device_descriptor = wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            label: Some("Device"),
            ..Default::default()
        };
        let (device, queue) = adapter.request_device(&device_descriptor).await.unwrap();

        let surface_capatibilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capatibilities
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_capatibilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_capatibilities.present_modes[0],
            alpha_mode: surface_capatibilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        Ctx {
            surface,
            config,
            device,
            queue,
            size,
            // window,
        }
    }

    pub fn new(window: Arc<Window>) -> Ctx<'ctx> {
        block_on(Ctx::async_new(window))
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.size = size;
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn draw(&mut self) -> Result<(), wgpu::SurfaceError> {
        let drawable = self.surface.get_current_texture()?;

        let image_view_descriptor = wgpu::TextureViewDescriptor::default();
        let image_view = drawable.texture.create_view(&image_view_descriptor);

        let command_encoder_descriptor = wgpu::CommandEncoderDescriptor {
            label: Some("Render encoder"),
        };
        let mut command_encoder = self
            .device
            .create_command_encoder(&command_encoder_descriptor);

        let color_attachment = wgpu::RenderPassColorAttachment {
            view: &image_view,
            resolve_target: None,
            depth_slice: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.75,
                    g: 0.5,
                    b: 0.25,
                    a: 1.,
                }),
                store: wgpu::StoreOp::Store,
            },
        };

        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[Some(color_attachment)],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        };

        command_encoder.begin_render_pass(&render_pass_descriptor);
        self.queue.submit(std::iter::once(command_encoder.finish()));

        drawable.present();

        Ok(())
    }
}
