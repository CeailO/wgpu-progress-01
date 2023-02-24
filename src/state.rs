use std::iter::once;
use std::path::PathBuf;

use wgpu::{
    Backends, BlendComponent, BlendState, Color, ColorTargetState, ColorWrites,
    CommandEncoderDescriptor, Device, DeviceDescriptor, Dx12Compiler, Face, Features,
    FragmentState, FrontFace, IndexFormat, Instance, InstanceDescriptor, Limits, MultisampleState,
    Operations, PipelineLayoutDescriptor, PolygonMode, PowerPreference, PrimitiveState,
    PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, ShaderModuleDescriptor, Surface,
    SurfaceConfiguration, SurfaceError, TextureUsages, TextureViewDescriptor, VertexState,
};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
pub struct State {
    surface: Surface,
    device: Device,
    queue: Queue,
    configuration: SurfaceConfiguration,
    size: PhysicalSize<u32>,
    window: Window,
    //
    render_pipeline: RenderPipeline,
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
        //
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
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
                buffers: &[],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::LineStrip, // LineList // Pointlist
                strip_index_format: Some(IndexFormat::Uint32), // None // None
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
        Self {
            surface,
            device,
            queue,
            configuration,
            size,
            window,
            //
            render_pipeline,
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
            render_pass.draw(0..6, 0..6) // 3
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
                    Err(SurfaceError::Timeout) => println!("Surface timeout"),
                }
            }
            Event::MainEventsCleared => {
                state.window().request_redraw();
            }
            _ => {}
        });
    }
}
