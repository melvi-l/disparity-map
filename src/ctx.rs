use std::sync::Arc;

use winit::{dpi::PhysicalSize, window::Window};

use crate::block::block_on;

pub struct Ctx<'ctx> {
    surface: wgpu::Surface<'ctx>,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: PhysicalSize<u32>,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl<'ctx> Ctx<'ctx> {
    async fn async_new(
        window: Arc<Window>,
        png_decoded_u16: &[u16],
        png_width: u32,
        png_height: u32,
    ) -> Self {
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

        let texture = Ctx::create_png_u16_texture(&device, png_width, png_height);

        Ctx::upload_png_u16_data_to_texture(
            &queue,
            &texture,
            &decoded_png_u16_to_grayscale_u8(png_decoded_u16),
            png_width,
            png_height,
        );

        let (bind_group_layout, bind_group) = Ctx::create_png_bind_group(&device, &texture);

        let pipeline = Ctx::create_png_render_pipeline(&device, bind_group_layout, config.format);

        Ctx {
            surface,
            config,
            device,
            queue,
            size,
            bind_group,
            pipeline,
        }
    }

    pub fn new(
        window: Arc<Window>,
        png_decoded_u16: &[u16],
        png_width: u32,
        png_height: u32,
    ) -> Ctx<'ctx> {
        block_on(Ctx::async_new(
            window,
            png_decoded_u16,
            png_width,
            png_height,
        ))
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
                    r: 1.,
                    g: 0.,
                    b: 0.,
                    a: 1.,
                }),
                store: wgpu::StoreOp::Store,
            },
        };

        {
            let render_pass_descriptor = wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(color_attachment)],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            };
            let mut render_pass = command_encoder.begin_render_pass(&render_pass_descriptor);
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }
        self.queue.submit(std::iter::once(command_encoder.finish()));
        drawable.present();

        Ok(())
    }
    pub fn create_png_u16_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Png u16 texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        texture
    }
    pub fn upload_png_u16_data_to_texture(
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        buffer: &[u8],
        width: u32,
        height: u32,
    ) {
        println!("{}", texture.size().width);
        queue.write_texture(
            wgpu::TexelCopyTextureInfoBase {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &buffer,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width),
                rows_per_image: Some(height),
            },
            texture.size(),
        );
    }

    pub fn create_png_bind_group(
        device: &wgpu::Device,
        texture: &wgpu::Texture,
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("PNG BindGroup Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("PNG Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("PNG BindGroup"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        (bind_group_layout, bind_group)
    }

    pub fn create_png_render_pipeline(
        device: &wgpu::Device,
        bind_group_layout: wgpu::BindGroupLayout,
        format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let shader_text = r#"
            struct VertexOutput {
                @builtin(position) pos: vec4<f32>,
                @location(0) uv: vec2<f32>,
            };

            @vertex
            fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
                var pos = array<vec2<f32>, 6>(
                    vec2<f32>(-1.0, -1.0),
                    vec2<f32>( 1.0, -1.0),
                    vec2<f32>(-1.0,  1.0),
                    vec2<f32>(-1.0,  1.0),
                    vec2<f32>( 1.0, -1.0),
                    vec2<f32>( 1.0,  1.0),
                );

                var uv = array<vec2<f32>, 6>(
                    vec2<f32>(0.0, 1.0),
                    vec2<f32>(1.0, 1.0),
                    vec2<f32>(0.0, 0.0),
                    vec2<f32>(0.0, 0.0),
                    vec2<f32>(1.0, 1.0),
                    vec2<f32>(1.0, 0.0),
                );

                var out: VertexOutput;
                out.pos = vec4<f32>(pos[idx], 0.0, 1.0);
                out.uv = uv[idx];
                return out;
            }

            @group(0) @binding(0) var tex: texture_2d<f32>;
            @group(0) @binding(1) var sam: sampler;

            @fragment
            fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
                let gray: f32 = textureSample(tex, sam, in.uv).r;
                return vec4<f32>(gray, gray, gray, 1.0);
            }
            "#;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("PNG Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_text.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("PNG Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("PNG Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        pipeline
    }
}

pub fn decoded_png_u16_to_grayscale_u8(buffer_u16: &[u16]) -> Vec<u8> {
    let min = *buffer_u16.iter().min().unwrap();
    let max = *buffer_u16.iter().max().unwrap();
    let range = (max - min).max(1);

    buffer_u16
        .iter()
        .map(|&v| (((v - min) as f32 / range as f32) * 255.0) as u8)
        .collect()
}
