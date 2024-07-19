use wgpu::{Backends, Instance, InstanceDescriptor, InstanceFlags};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

pub struct Simulation {
    instance: Instance,
}

impl Simulation {
    pub fn new() -> Simulation {
        let instance = init_wgpu();
        let mut simulation = Simulation { instance };
        simulation
    }
}

impl ApplicationHandler for Simulation {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        init_window(event_loop);
        println!("resumed!")
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::Destroyed { .. } => println!("Destroyed"),
            _ => {}
        }
    }
}

fn init_wgpu() -> Instance {
    let instance_descriptor = InstanceDescriptor {
        backends: Backends::VULKAN,
        flags: InstanceFlags::empty(),
        dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
        gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
    };
    Instance::new(instance_descriptor)
}

fn init_window(event_loop: &ActiveEventLoop) -> Window {
    let window_attributes = Window::default_attributes().with_title("Plant Simulation");
    event_loop
        .create_window(window_attributes)
        .expect("Window creation for winit failed.")
}
