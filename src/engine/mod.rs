use std::sync::{Arc, Mutex};

use camera::CameraController;
use cell::{CellRenderer, Size};
use futures::executor::block_on;
use state::ApplicationState;
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop,
    window::Window,
};

use crate::{
    model::{cell::Cell, entity::Entity},
    SimulationEvent,
};

mod camera;
pub mod cell;
mod state;
mod vertex;

const LEVEL_OF_DETAIL: u16 = 10;

pub struct Simulation<'w> {
    cells: Arc<Mutex<Vec<(Cell, CellRenderer)>>>,
    window: Option<Arc<Window>>,
    camera_controller: Arc<Mutex<CameraController>>,
    state: Option<ApplicationState<'w>>,
}

impl<'w> Simulation<'w> {
    pub fn new(cells: Vec<Cell>) -> Self {
        let mut cells_with_renderers = Vec::new();
        for cell in cells.into_iter() {
            let volume = (&cell).volume();
            let position = (&cell).position();
            cells_with_renderers.push((
                cell,
                CellRenderer::new(Size::FromVolume(volume), position, LEVEL_OF_DETAIL),
            ));
        }
        let simulation = Simulation {
            cells: Arc::new(Mutex::new(cells_with_renderers)),
            window: None,
            state: None,
            camera_controller: Arc::new(Mutex::new(CameraController::new(0.2))),
        };
        simulation
    }

    fn render(&self, state: &ApplicationState<'w>) {
        state.render().unwrap();
    }

    pub fn update(&mut self) {
        for (cell, cell_renderer) in self.cells.lock().unwrap().iter_mut() {
            cell.update();
            cell_renderer.update_size(Size::FromVolume(cell.volume()), LEVEL_OF_DETAIL);
        }
        match &self.state {
            None => {}
            Some(state) => {
                self.render(state);
            }
        };
    }
}

impl<'w> ApplicationHandler<SimulationEvent> for Simulation<'w> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(init_window(event_loop));
        self.window = Some(window.clone());
        let cells = Arc::clone(&self.cells);
        let state = block_on(ApplicationState::new(
            window,
            cells,
            self.camera_controller.clone(),
        ));
        self.state = Some(state);
        println!("resumed!");
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: SimulationEvent) {
        match event {
            SimulationEvent::Update => {
                self.update();
            }
        }
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
