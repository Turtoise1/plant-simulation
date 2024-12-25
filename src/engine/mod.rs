use std::sync::{Arc, Mutex, RwLock};

use camera::CameraController;
use cell_renderer::Size;
use futures::executor::block_on;
use state::ApplicationState;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::ActiveEventLoop,
    window::Window,
};

use crate::{model::entity::Entity, shared::cell::Cell, SimulationEvent};

mod camera;
pub mod cell_renderer;
mod delaunay;
mod state;
mod vertex;

const LEVEL_OF_DETAIL: u16 = 50;

pub struct Simulation<'w> {
    cells: Arc<RwLock<Vec<Cell>>>,
    window: Option<Arc<Window>>,
    camera_controller: Arc<Mutex<CameraController>>,
    state: Option<ApplicationState<'w>>,
}

impl<'w> Simulation<'w> {
    pub fn new(cells: Vec<Cell>) -> Self {
        let simulation = Simulation {
            cells: Arc::new(RwLock::new(cells)),
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
            // let tetraeders = delaunay_triangulation(&self.cells).unwrap();
            // println!("{:?}", tetraeders);
            let mut cell_write_guard = self.cells.write().unwrap();
            for cell in cell_write_guard.iter_mut() {
                {
                    let mut bio_write_guard = cell.bio.write().unwrap();
                    bio_write_guard.update();
                }
                {
                    let bio_read_guard = cell.bio.read().unwrap();
                    let mut renderer_write_guard = cell.renderer.write().unwrap();
                    renderer_write_guard.update(
                        Size::FromVolume(bio_read_guard.volume().clone()),
                        LEVEL_OF_DETAIL,
                        &vec![],
                    );
                }
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

// fn get_other_cells(cells: &MutexGuard<'_, Vec<Cell>>, cell: &Cell) -> Vec<Cell> {
//     let other_cells: Vec<Cell> = cells
//         .iter()
//         .filter(|other_cell| other_cell.entity_id() != cell.entity_id())
//         .map(|other_cell| other_cell.clone()) // Clone the cell here
//         .collect();
//     other_cells
// }

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
                let camera_controller = Arc::clone(&self.camera_controller);
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
            WindowEvent::MouseInput { button, state, .. } => match button {
                MouseButton::Left => match state {
                    ElementState::Released => {
                        let position = self
                            .state
                            .as_ref()
                            .unwrap()
                            .mouse_position
                            .as_ref()
                            .unwrap();
                        println!("Clicked at {:?}!", position);
                        // TODO: if a cell has been hit with this position, set the cell as acive and use its center as camera center.
                    }
                    _ => {}
                },
                _ => {}
            },
            WindowEvent::MouseWheel { delta, phase, .. } => {
                println!("Got mouse wheel event: {:?}, {:?}", delta, phase);
                let camera_controller = Arc::clone(&self.camera_controller);
                let _ = camera_controller
                    .lock()
                    .as_mut()
                    .unwrap()
                    .process_mouse_wheel(&delta, &phase);
                let state = self.state.as_mut().unwrap();
                state.update_camera();
                let state = self.state.as_ref().unwrap();
                self.render(state);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.state.as_mut().unwrap().mouse_position = Some(position);
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
