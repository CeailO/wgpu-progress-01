use std::{iter::once, num::NonZeroU32};

use std::path::PathBuf;

use bytemuck::cast_slice;
use image::{load_from_memory, GenericImageView};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Backends, BlendComponent, BlendState, Buffer, BufferUsages, Color, ColorTargetState,
    ColorWrites, CommandEncoderDescriptor, Device, DeviceDescriptor, Dx12Compiler, Extent3d,
    Features, FragmentState, ImageCopyTexture, ImageDataLayout, IndexFormat, Instance,
    InstanceDescriptor, Limits, MultisampleState, Operations, Origin3d, PipelineLayoutDescriptor,
    PowerPreference, PrimitiveState, PrimitiveTopology, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions,
    ShaderModuleDescriptor, Surface, SurfaceConfiguration, SurfaceError, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor, VertexState,
};
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, FilterMode, SamplerDescriptor,
    ShaderStages, TextureSampleType, TextureViewDimension,
};

use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::vertex_lib::vertex_lib::{Vertex, INDICES, VERTICES};

pub struct State {
    surface: Surface,
    device: Device,
    queue: Queue,
    configuration: SurfaceConfiguration,
    size: PhysicalSize<u32>,
    window: Window,
    //
    render_pipeline: RenderPipeline,
    // alt-3
    vertex_buffer: Buffer,
    num_vertices: u32,
    // alt-4
    index_buffer: Buffer,
    num_indices: u32,
    // alt-5
    diffuse_bind_group: BindGroup,
}

impl State {
    pub async fn new(window: Window) -> Self {
        //
        let size = window.inner_size();
        //
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::DX12,
            dx12_shader_compiler: Dx12Compiler::Dxc {
                dxil_path: Some(PathBuf::from("/dxc_2022_12_16/bin/x64/dxcompiler.dll")),
                dxc_path: Some(PathBuf::from("/dxc_2022_12_16/bin/x64/dxil.dll")),
            },
        });

        let report = instance.generate_report();
        dbg!(&report);

        //
        let surface = unsafe { instance.create_surface(&window) }.unwrap();
        //
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        //
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    features: Features::empty(),
                    limits: Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();
        //
        let surface_caps = surface.get_capabilities(&adapter);
        //
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.describe().srgb)
            .next()
            .unwrap_or(surface_caps.formats[0]);
        //
        let configuration = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &configuration);

        // ----- CHANGES -----
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        // vertex buffer
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: cast_slice(VERTICES),
            usage: BufferUsages::VERTEX,
        });
        //
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vertex_shader_main",
                // buffer from vertex_buffer
                buffers: &[Vertex::desc()],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList, // LineList // Pointlist
                strip_index_format: None,                  // None // None
                ..Default::default()
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: configuration.format,
                    blend: Some(BlendState {
                        color: BlendComponent::REPLACE, //
                        alpha: BlendComponent::REPLACE, //
                    }),
                    write_mask: (ColorWrites::ALL),
                })],
            }),
            multiview: None,
        });
        //
        let num_vertices = VERTICES.len() as u32;

        // ----- CHANGES -----
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: cast_slice(INDICES),
            usage: BufferUsages::INDEX,
        });
        let num_indices = INDICES.len() as u32;

        // ----- CHANGES -----
        let diffuse_bytes = include_bytes!(r"assets\typescript.png");
        let diffuse_image = load_from_memory(diffuse_bytes).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();

        let dimensions = diffuse_image.dimensions();

        let texture_size = Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let diffuse_texture = device.create_texture(&TextureDescriptor {
            label: Some("Diffuse Texture"),
            size: texture_size, // stored as 3d representing as 2d
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb, // most images stored using sRGB
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[], // Usage others than Rgba8UnormSrgb is not supperted by WebGL2 backend
        });

        // ----- Load data into a texture -----
        // where to copy gpu data
        queue.write_texture(
            ImageCopyTexture {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &diffuse_rgba, // the actual pixel data
            ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * dimensions.0),
                rows_per_image: NonZeroU32::new(dimensions.1),
            },
            texture_size,
        );

        let diffuse_texture_view = diffuse_texture.create_view(&TextureViewDescriptor::default());
        let diffuse_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Diffuse Sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(wgpu::SamplerBindingType::Filtering), // match filterable field icorresponding to the first entries index
                        count: None,
                    },
                ],
            });

        let diffuse_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Diffuse Bind Group"),
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&diffuse_texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&diffuse_sampler),
                },
            ],
        });

        Self {
            surface,
            device,
            queue,
            configuration,
            size,
            window,
            //
            render_pipeline,
            //
            vertex_buffer,
            num_vertices,
            index_buffer,
            num_indices,
            //
            diffuse_bind_group,
        }
    }

    fn window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.size = size;
            self.configuration.width = size.width;
            self.configuration.height = size.height;
            self.surface.configure(&self.device, &self.configuration);
        }
    }

    #[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        // self.window.request_redraw();
    }

    fn render(&mut self) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: wgpu::LoadOp::Clear(Color {
                            r: 0.05,
                            g: 0.062,
                            b: 0.08,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            //
            render_pass.set_pipeline(&self.render_pipeline); // 2
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }
        self.queue.submit(once(encoder.finish()));
        output.present();
        Ok(())
    }

    pub async fn run() {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        let mut state = State::new(window).await;

        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    Err(SurfaceError::Lost | SurfaceError::Outdated) => state.resize(state.size),
                    Err(SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(SurfaceError::Timeout) => println!("Surface Timeout"),
                }
            }
            Event::MainEventsCleared => {
                state.window().request_redraw();
            }
            _ => {}
        });
    }
}
