use std::sync::{Arc, Mutex, MutexGuard, Weak};

use camera::CameraController;
use cell_renderer::Size;
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
pub mod cell_renderer;
mod state;
mod vertex;

const LEVEL_OF_DETAIL: u16 = 30;

pub struct Simulation<'w> {
    cells: Arc<Mutex<Vec<Arc<Mutex<Cell>>>>>,
    window: Option<Arc<Window>>,
    camera_controller: Arc<Mutex<CameraController>>,
    state: Option<ApplicationState<'w>>,
}

impl<'w> Simulation<'w> {
    pub fn new(cells: Vec<Arc<Mutex<Cell>>>) -> Self {
        let simulation = Simulation {
            cells: Arc::new(Mutex::new(cells)),
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
        {
            let cell_refs: Vec<_> = self
                .cells
                .lock()
                .unwrap()
                .iter()
                .map(|cell| Arc::clone(cell))
                .collect();
            for cell in self.cells.lock().unwrap().iter() {
                // Create a filtered Vec of the other cells
                let other_cells = cell_refs
                    .iter()
                    .filter_map(|other_cell| {
                        let other_id;
                        {
                            other_id = other_cell.lock().unwrap().get_entity_id().clone();
                        }
                        if other_id != cell.lock().unwrap().get_entity_id() {
                            Some(Arc::clone(other_cell))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                let mut cell = cell.lock().unwrap();
                cell.update();
                match &cell.renderer {
                    Some(renderer) => {
                        let mut renderer = renderer.lock().unwrap();
                        renderer.update(
                            Size::FromVolume(cell.volume()),
                            LEVEL_OF_DETAIL,
                            other_cells,
                        );
                    }
                    None => {
                        println!("Renderer not initialized!");
                    }
                };
            }
        }
        match &self.state {
            None => {}
            Some(state) => {
                self.render(state);
            }
        };
    }
}

fn get_other_cells(cells: &MutexGuard<'_, Vec<Cell>>, cell: &Cell) -> Vec<Cell> {
    let other_cells: Vec<Cell> = cells
        .iter()
        .filter(|other_cell| other_cell.get_entity_id() != cell.get_entity_id())
        .map(|other_cell| other_cell.clone()) // Clone the cell here
        .collect();
    other_cells
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
