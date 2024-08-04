use std::{
    borrow::BorrowMut,
    sync::{Arc, Mutex},
};

use camera::CameraController;
use cell::CellRenderer;
use futures::executor::block_on;
use state::ApplicationState;
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop,
    window::Window,
};

mod camera;
pub mod cell;
mod state;
mod vertex;

pub struct Simulation<'w> {
    cells: Arc<Mutex<Vec<CellRenderer>>>,
    window: Option<Arc<Window>>,
    camera_controller: Arc<Mutex<CameraController>>,
    state: Option<ApplicationState<'w>>,
}

impl<'w> Simulation<'w> {
    pub fn new() -> Self {
        let simulation = Simulation {
            cells: Arc::new(Mutex::new(Vec::new())),
            window: None,
            state: None,
            camera_controller: Arc::new(Mutex::new(CameraController::new(0.2))),
        };
        let clone = simulation.cells.clone();
        let mut cells = clone.lock().unwrap();
        cells.push(CellRenderer::new(1., [0., 0., 0.], 10));
        simulation
    }

    fn render(&self, state: &ApplicationState<'w>) {
        state.render().unwrap();
        //let vertex_buffer = state.get_vertex_buffer();
        //state.draw(&render_pipeline, &vertex_buffer);
    }
}

impl<'w> ApplicationHandler for Simulation<'w> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(init_window(event_loop));
        self.window = Some(window.clone());
        let clone = self.cells.clone();
        let state = block_on(ApplicationState::new(
            window,
            clone,
            self.camera_controller.clone(),
        ));
        self.state = Some(state);
        println!("resumed!");
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::Destroyed { .. } => println!("Destroyed"),
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested { .. } => {
                let state = self.state.as_mut().unwrap();
                state.update_camera();
                let state = self.state.as_ref().unwrap();
                self.render(state);

                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                // self.window.as_ref().unwrap().clone().request_redraw();
            }
            WindowEvent::Resized { .. } => {
                let state = self.state.as_mut().unwrap();
                state.resize();
                state.update_camera();
                let state = self.state.as_ref().unwrap();
                self.render(state);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let camera_controller = self.camera_controller.clone();
                let _ = camera_controller
                    .lock()
                    .as_mut()
                    .unwrap()
                    .process_events(&event);
                let state = self.state.as_mut().unwrap();
                state.update_camera();
                let state = self.state.as_ref().unwrap();
                self.render(state);
            }
            _ => {}
        }
    }
}

fn init_window(event_loop: &ActiveEventLoop) -> Window {
    let window_attributes = Window::default_attributes().with_title("Plant Simulation");
    event_loop
        .create_window(window_attributes)
        .expect("Window creation for winit failed.")
}
