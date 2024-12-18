use encase::{ShaderType};
use glam::{vec2, Vec2};
use std::sync::Arc;
use std::time::Instant;
use wgpu::{BindGroup, Buffer, Device, Queue, RenderPipeline, Surface, SurfaceConfiguration};
use winit::application::ApplicationHandler;
use winit::event::*;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use crate::fixed_buf::DoubleBuf;

type MyBuf = DoubleBuf<100, 100>;

// Uniform buffer.
#[derive(Debug, ShaderType, Default)] // this baby can fit so many derive macros
struct State {
    pub cursor_pos: glam::Vec2,
    pub dimensions: glam::Vec2,
    pub time: f32,
    //pub buffer_handle: BufferHandle
}

impl State {
    fn as_wgsl_bytes(&self) -> encase::internal::Result<Vec<u8>> {
        let mut buffer = encase::UniformBuffer::new(Vec::new());
        buffer.write(self)?;
        Ok(buffer.into_inner())
    }


}

// UHH NOT THE STATE
// https://www.youtube.com/watch?v=rGV0E7f8zeg
struct Context<'a> {
    config: SurfaceConfiguration,
    surface: Surface<'a>,
    device: Device,
    render_pipeline: RenderPipeline,
    queue: Queue,
    bind_group: BindGroup,
    uniform_buffer: Buffer
}

// https://github.com/rust-windowing/winit/discussions/3667#discussioncomment-9329312
impl<'a> Context<'a> {
    pub async fn new(window: Arc<Window>) -> Context<'a> {
        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);

        let instance = wgpu::Instance::default();

        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                },
                None,
            )
            .await
            .expect("Failed to create device");

        // Load the shaders from disk
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "shader.wgsl"
            ))),
        });

        // https://github.com/gfx-rs/wgpu/blob/trunk/examples/src/uniform_values/mod.rs
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("bbabby first uniform"),
            size: State::min_size().into(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("mah bind group"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // holy boilerplate
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        surface.configure(&device, &config);

        Context {
            config,
            surface,
            device,
            render_pipeline,
            queue,
            uniform_buffer,
            bind_group
        }
    }
}

pub struct App<'a> {
    window: Option<Arc<Window>>, // AHHH I SEE, ARCS ARE TAXATION
    ctx: Option<Context<'a>>,
    state: State,
    start: std::time::Instant
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        let window = None;
        let ctx = None;
        let state = State::default();
        let start = Instant::now();
        Self { window, ctx, state, start }
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            self.start = Instant::now();
            let window = Arc::new(
                event_loop
                    .create_window(Window::default_attributes())
                    .unwrap(),
            );
            self.window = Some(window.clone());

            let state = pollster::block_on(Context::new(window.clone()));
            self.ctx = Some(state);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let Some(state) = self.ctx.as_mut() {
                    // Reconfigure the surface with the new size
                    state.config.width = new_size.width.max(1);
                    state.config.height = new_size.height.max(1);
                    state.surface.configure(&state.device, &state.config);
                    // Also update the uniform.
                    self.state.dimensions = vec2(new_size.width as f32, new_size.height as f32);

                    // On macos the window needs to be redrawn manually after resizing
                    self.window.as_ref().unwrap().request_redraw();
                }
            },
            WindowEvent::CursorMoved { device_id, position } => {
                self.state.cursor_pos = Vec2{ x: position.x as f32, y: position.y as f32 };
                self.window.as_ref().unwrap().request_redraw();
            },
            WindowEvent::RedrawRequested => {
                let elapsed = self.start.elapsed();
                self.state.time = elapsed.as_secs_f32();

                if let Some(ctx) = self.ctx.as_ref() {
                    let frame = ctx
                        .surface
                        .get_current_texture()
                        .expect("Failed to acquire next swap chain texture");
                    let view = frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());

                    ctx.queue.write_buffer(&ctx.uniform_buffer, 0, &self.state.as_wgsl_bytes().expect("uhh"));
                    println!("Uniform: {:?}", &self.state);

                    let mut encoder = ctx
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                    {
                        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: None,
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        });
                        rpass.set_pipeline(&ctx.render_pipeline);
                        rpass.set_bind_group(0, Some(&ctx.bind_group), &[]);
                        // NB: Here's where we specify the indices to render, these are defined in the shader.
                        rpass.draw(0..6, 0..1);
                    }

                    ctx.queue.submit(Some(encoder.finish()));
                    frame.present();
                }
            }
            _ => (),
        }
    }
}
