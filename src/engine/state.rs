use std::sync::{Arc, Mutex};

use cgmath::{EuclideanSpace, InnerSpace, Point3, SquareMatrix, Vector3, Vector4};
use wgpu::{
    util::DeviceExt, Adapter, Backends, Buffer, Device, Instance, InstanceDescriptor,
    InstanceFlags, MemoryHints, Queue, RenderPipeline, Surface, TextureFormat,
};
use winit::{dpi::PhysicalPosition, window::Window};

use crate::shared::{
    cell::{Cell, CellEvent, CellEventType, EventSystem},
    math::{distance, line_plane_intersection, Line, Line2PlaneClassification, Plane},
};

use super::{
    camera::{Camera, CameraController, CameraUniform},
    vertex::Vertex,
};

pub struct ApplicationState<'window> {
    window: Arc<Window>,
    adapter: Adapter,
    surface: Surface<'window>,
    device: Device,
    queue: Queue,
    render_pipeline: Option<RenderPipeline>,
    cells: Arc<Vec<Cell>>,
    cell_events: Arc<EventSystem>,
    pub mouse_position: Option<PhysicalPosition<f64>>,
    camera: Camera,
    camera_controller: Arc<Mutex<CameraController>>,
    camera_uniform: CameraUniform,
    camera_buffer: Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_bind_group_layout: wgpu::BindGroupLayout,
}

impl<'window> ApplicationState<'window> {
    pub async fn new(
        window: Arc<Window>,
        cells: Arc<Vec<Cell>>,
        cell_events: Arc<EventSystem>,
        camera_controller: Arc<Mutex<CameraController>>,
    ) -> Self {
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

        let size = window.as_ref().inner_size();
        let camera = Camera {
            // position the camera 0 units up and 3 units back
            // +z is out of the screen
            eye: (0.0, 0.0, 3.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: cgmath::Vector3::unit_y(),
            aspect: size.width as f32 / size.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let mut state = ApplicationState {
            window,
            adapter,
            surface,
            device,
            queue,
            render_pipeline: None,
            cells,
            cell_events,
            mouse_position: None,
            camera,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_bind_group_layout,
        };
        let render_pipeline = state.get_render_pipeline();
        state.render_pipeline = Some(render_pipeline);
        state.resize();
        state
    }

    pub fn screen_pos_2_select_ray(&self, screen_pos: &PhysicalPosition<f64>) -> Line<f32> {
        let view_projection_matrix = self.camera.build_view_projection_matrix();
        let inverted = view_projection_matrix.invert().unwrap();
        let width = self.window.inner_size().width as f64;
        let height = self.window.inner_size().height as f64;
        let x = screen_pos.x;
        let y = screen_pos.y;
        // Convert screen position to NDC
        let ndc_x = ((2.0 * x) / width - 1.0) as f32;
        let ndc_y = (1.0 - (2.0 * y) / height) as f32;

        // Clip space coordinates
        let near_clip = Vector4::new(ndc_x, ndc_y, -1.0, 1.0);
        let far_clip = Vector4::new(ndc_x, ndc_y, 1.0, 1.0);

        // Transform to world space
        let near_world = inverted * near_clip;
        let far_world = inverted * far_clip;

        // Perspective divide to get 3D coordinates
        let near_point = Point3::new(
            near_world.x / near_world.w,
            near_world.y / near_world.w,
            near_world.z / near_world.w,
        );

        let far_point = Point3::new(
            far_world.x / far_world.w,
            far_world.y / far_world.w,
            far_world.z / far_world.w,
        );

        // Compute ray direction
        let ray_dir = (far_point - near_point).normalize();
        Line {
            pos: near_point.to_vec(),
            dir: ray_dir,
        }
    }

    pub fn select_cells(&self, select_ray: Line<f32>) {
        for cell in self.cells.iter() {
            let renderer = cell.renderer.read().unwrap();
            let cell_pos = renderer.position();
            let cell_pos = Vector3::new(cell_pos.x, cell_pos.y, cell_pos.z);
            let cell_plane = Plane::<f32> {
                pos: cell_pos,
                normal: select_ray.dir,
            };
            match line_plane_intersection(&select_ray, &cell_plane) {
                Line2PlaneClassification::Parallel => {
                    panic!("This should not happen because the plane should be orthogonal to the select ray!")
                }
                Line2PlaneClassification::Intersects(intersection_point) => {
                    if distance(
                        &(Point3::origin() + cell_pos),
                        &(Point3::origin() + intersection_point),
                    ) < *renderer.radius()
                    {
                        println!("Intersection with cell {}", renderer.cell_id());
                        self.cell_events.notify(Arc::new(CellEvent {
                            id: renderer.cell_id(),
                            event_type: CellEventType::Mark(Option::None),
                        }));
                    }
                }
            }
        }
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // optionally splitting into parts, maybe useful for multithreading i don't know
        // let parts = split_into_parts(&cells, 8);
        let parts = &[&self.cells]; // instead only one part at the moment

        let mut encoders = Vec::new();
        for (index, cells) in parts.iter().enumerate() {
            encoders.push(self.encode_cells(&view, &cells, index == 0).finish());
        }
        self.queue.submit(encoders.into_iter());

        output.present();

        Ok(())
    }

    fn encode_cells(
        &self,
        view: &wgpu::TextureView,
        cells: &Vec<Cell>,
        first: bool,
    ) -> wgpu::CommandEncoder {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let load_operation = match first {
                true => wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.1,
                    g: 0.2,
                    b: 0.3,
                    a: 1.0,
                }),
                false => wgpu::LoadOp::Load,
            };
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: load_operation,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            for cell in cells.iter() {
                let renderer = cell.renderer.read().unwrap();
                let vertices = renderer.vertices();
                let indices = renderer.indices();
                let vertex_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Vertex Buffer"),
                            contents: bytemuck::cast_slice(vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
                let index_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Index Buffer"),
                            contents: bytemuck::cast_slice(indices),
                            usage: wgpu::BufferUsages::INDEX,
                        });
                let num_indices = indices.len() as u32;
                render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                render_pass.set_pipeline(&self.render_pipeline.as_ref().unwrap());
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..num_indices, 0, 0..1);
            }
        };
        encoder
    }

    pub fn resize(&mut self) {
        let size = self.window.as_ref().inner_size();
        let config =
            Surface::get_default_config(&self.surface, &self.adapter, size.width, size.height)
                .expect("Could not get default configuration for the surface.");
        self.surface.configure(&self.device, &config);
        let camera = Camera {
            // position the camera 0 unit up and 3 units back
            // +z is out of the screen
            eye: (0.0, 0.0, 3.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: cgmath::Vector3::unit_y(),
            aspect: size.width as f32 / size.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };
        self.camera = camera;
    }

    pub fn update_camera(&mut self) {
        self.camera_controller
            .lock()
            .as_ref()
            .unwrap()
            .update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    fn get_render_pipeline(&self) -> RenderPipeline {
        // Create the shader modules
        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            });

        let render_pipeline_layout =
            self.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&self.camera_bind_group_layout],
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
                    buffers: &[Vertex::desc()],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main", // fragment function name entry point from shader.wgsl
                    targets: &[Some(wgpu::ColorTargetState {
                        format: TextureFormat::Bgra8UnormSrgb,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                        }),
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

fn split_into_parts<T>(array: &[T], num_parts: usize) -> Vec<&[T]> {
    let len = array.len();
    let mut parts = Vec::new();
    let mut start = 0;

    for i in 0..num_parts {
        // Calculate the size of each part.
        // Distribute any remainder to the first few parts.
        let part_size = (len + i) / num_parts;
        let end = start + part_size;
        parts.push(&array[start..end]);
        start = end;
    }

    parts
}
