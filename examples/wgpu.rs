/// This example demonstrates how to draw a 2D glyph using wgpu.
///
/// Please note that the character would look better with anti-aliasing,
/// but in order to keep the sample simple this was omitted.
///
use meshtext::{Glyph, MeshGenerator, MeshText};
use std::{borrow::Cow, sync::Arc};
use wgpu::{util::DeviceExt, MemoryHints};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

const SHADER: &str = r##"
@vertex
fn vs_main(@location(0) position: vec2<f32>) -> @builtin(position) vec4<f32> {
    let scale = vec3<f32>(2.0, 2.0, 2.0);
    return vec4<f32>(position.x * scale.x - 0.5, position.y * scale.y - 0.5, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
"##;

struct InstanceData {
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    surface: wgpu::Surface<'static>,
    vertex_buffer: wgpu::Buffer,
}

struct App {
    instance_data: Option<InstanceData>,
    vertex_count: u32,
    vertex_data: Vec<u8>,
    window: Option<Arc<Window>>,
    window_attributes: WindowAttributes,
}

impl App {
    async fn initialize(&mut self, window: &Arc<Window>) {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            backend_options: wgpu::BackendOptions {
                dx12: wgpu::Dx12BackendOptions {
                    shader_compiler: wgpu::Dx12Compiler::Fxc,
                    ..Default::default()
                },
                gl: wgpu::GlBackendOptions {
                    gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        });
        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create surface.");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                memory_hints: MemoryHints::Performance,
                ..Default::default()
            })
            .await
            .expect("Failed to create device");

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(SHADER)),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = *capabilities
            .formats
            .first()
            .expect("Surface does not support any texture format.");

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: self.vertex_data.as_slice(),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<f32>() as wgpu::BufferAddress * 2,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let config = wgpu::SurfaceConfiguration {
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            format: swapchain_format,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: vec![],
            width: size.width,
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        self.instance_data = Some(InstanceData {
            config,
            device,
            queue,
            render_pipeline,
            surface,
            vertex_buffer,
        });
    }

    fn new(vertex_count: u32, vertex_data: Vec<u8>, window_attributes: WindowAttributes) -> Self {
        App {
            instance_data: None,
            vertex_count,
            vertex_data,
            window: None,
            window_attributes,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = Arc::new(
                event_loop
                    .create_window(self.window_attributes.clone())
                    .unwrap(),
            );

            pollster::block_on(self.initialize(&window));

            self.window = Some(window);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::Resized(size) => {
                if let Some(instance) = &mut self.instance_data {
                    instance.config.width = size.width;
                    instance.config.height = size.height;
                    instance
                        .surface
                        .configure(&instance.device, &instance.config);
                }

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(instance) = &self.instance_data {
                    let frame = instance
                        .surface
                        .get_current_texture()
                        .expect("Failed to acquire next swap chain texture");
                    let view = frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    let mut encoder = instance
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                    {
                        let mut render_pass =
                            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: None,
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &view,
                                    depth_slice: None,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                                        store: wgpu::StoreOp::Store,
                                    },
                                })],
                                depth_stencil_attachment: None,
                                occlusion_query_set: None,
                                timestamp_writes: None,
                            });
                        render_pass.set_vertex_buffer(0, instance.vertex_buffer.slice(..));
                        render_pass.set_pipeline(&instance.render_pipeline);
                        render_pass.draw(0..self.vertex_count, 0..1);
                    }

                    instance.queue.submit(Some(encoder.finish()));
                    frame.present();
                }
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {}
        }
    }
}

fn get_vertices_for_a() -> Vec<f32> {
    let character = 'A';
    let font_data = include_bytes!("../assets/font/FiraMono-Regular.ttf");
    let mut generator = MeshGenerator::new(font_data);
    let result: MeshText = generator
        .generate_glyph_2d(character, None)
        .expect("Failed to generate glyph.");

    result.vertices
}

fn main() {
    // Create the vertices for the letter "A".
    let text_vertices = get_vertices_for_a();
    let mut raw_data: Vec<u8> = Vec::new();
    for vert in text_vertices.iter() {
        raw_data.extend_from_slice(vert.to_le_bytes().as_slice());
    }

    // Render the created vertices using wgpu.
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let window_attributes =
        Window::default_attributes().with_inner_size(LogicalSize::new(600, 600));

    let mut app = App::new(text_vertices.len() as u32 / 2, raw_data, window_attributes);
    event_loop.run_app(&mut app).unwrap();
}
