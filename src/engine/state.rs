use std::sync::Arc;

use wgpu::{
    Adapter, Backends, Device, Instance, InstanceDescriptor, InstanceFlags, MemoryHints, Queue,
    RenderPipeline, Surface, TextureFormat,
};
use winit::window::Window;

pub struct ApplicationState<'window> {
    window: Arc<Window>,
    adapter: Adapter,
    surface: Surface<'window>,
    device: Device,
    queue: Queue,
    render_pipeline: Option<RenderPipeline>,
}

impl<'window> ApplicationState<'window> {
    pub async fn new(window: Arc<Window>) -> Self {
        let instance = create_instance();
        let surface = instance
            .create_surface(Arc::clone(&window))
            .expect("Surface creation in window not successful.");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Adapter creation failed!");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: MemoryHints::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();
        let mut state = ApplicationState {
            window,
            adapter,
            surface,
            device,
            queue,
            render_pipeline: None,
        };
        let render_pipeline = state.get_render_pipeline();
        state.render_pipeline = Some(render_pipeline);
        state.resize();
        state
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline.as_ref().unwrap()); // 2.
            render_pass.draw(0..3, 0..1); // 3.
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn resize(&self) {
        let size = self.window.as_ref().inner_size();
        let config =
            Surface::get_default_config(&self.surface, &self.adapter, size.width, size.height)
                .expect("Could not get default configuration for the surface.");
        self.surface.configure(&self.device, &config);
    }

    fn get_render_pipeline(&self) -> RenderPipeline {
        // Create the shader modules
        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            });

        // Define the pipeline layout and render pipeline
        let render_pipeline_layout =
            self.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });

        let render_pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main", // vertex function name entry point from shader.wgsl
                    buffers: &[],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main", // fragment function name entry point from shader.wgsl
                    targets: &[Some(wgpu::ColorTargetState {
                        format: TextureFormat::Bgra8UnormSrgb,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        render_pipeline
    }
}

fn create_instance() -> Instance {
    let instance_descriptor = InstanceDescriptor {
        backends: Backends::VULKAN,
        flags: InstanceFlags::empty(),
        dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
        gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
    };
    Instance::new(instance_descriptor)
}
