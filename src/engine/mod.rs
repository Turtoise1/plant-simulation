use std::{
    f32::consts::PI,
    sync::{Arc, Mutex},
    thread,
};

use camera::CameraController;
use cell::CellRenderer;
use futures::executor::block_on;
use state::ApplicationState;
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop,
    window::Window,
};

use crate::model::{cell::Cell, entity::Entity};

mod camera;
pub mod cell;
mod state;
mod vertex;

pub struct Simulation<'w> {
    cells: Vec<Cell>,
    cell_renderers: Vec<CellRenderer>,
    window: Option<Arc<Window>>,
    camera_controller: Arc<Mutex<CameraController>>,
    state: Option<ApplicationState<'w>>,
}

impl<'w> Simulation<'w> {
    pub fn new(cells: Vec<Cell>, level_of_detail: u16) -> Self {
        let mut cell_renderers = Vec::new();
        for cell in cells.iter() {
            // r = ((3V)/(4PI))^(1/3)
            let radius = f32::powf((3. * cell.volume()) / (4. * PI), 1. / 3.);
            cell_renderers.push(CellRenderer::new(radius, cell.position(), level_of_detail));
        }
        let simulation = Simulation {
            cells,
            cell_renderers,
            window: None,
            state: None,
            camera_controller: Arc::new(Mutex::new(CameraController::new(0.2))),
        };
        simulation
    }

    fn render(&self, state: &ApplicationState<'w>) {
        state.render().unwrap();
        //let vertex_buffer = state.get_vertex_buffer();
        //state.draw(&render_pipeline, &vertex_buffer);
    }

    fn update(&mut self, state: &ApplicationState<'w>) {
        for cell in self.cells.iter_mut() {
            cell.update();
        }
        self.render(state);
    }
}

impl<'w> ApplicationHandler for Simulation<'w> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(init_window(event_loop));
        self.window = Some(window.clone());
        let state = block_on(ApplicationState::new(
            window,
            self.cell_renderers.clone(),
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
