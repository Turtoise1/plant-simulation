use std::{
    borrow::BorrowMut,
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex, MutexGuard},
};

use camera::CameraController;
use cell_renderer::{CellRenderer, Size};
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
    cells: Arc<Mutex<Vec<Cell>>>,
    window: Option<Arc<Window>>,
    camera_controller: Arc<Mutex<CameraController>>,
    state: Option<ApplicationState<'w>>,
}

impl<'w> Simulation<'w> {
    pub fn new(cells: Vec<Cell>) -> Self {
        let cells_clone = cells.clone();
        let cells = cells
            .into_iter()
            .map(|mut cell| {
                let volume = (&cell).volume();
                let position = (&cell).position();
                let other_cells = cells_clone
                    .iter()
                    .filter(|other_cell| other_cell.get_entity_id() != cell.get_entity_id())
                    .map(|c| c.clone())
                    .collect();
                let renderer = CellRenderer::new(
                    Arc::new(RefCell::new(&cell)),
                    Size::FromVolume(volume),
                    position,
                    LEVEL_OF_DETAIL,
                    other_cells,
                );
                cell.set_renderer(Option::Some(Rc::downgrade(&Rc::new(RefCell::new(
                    renderer,
                )))));
                cell
            })
            .collect();
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
            let mut cells = self.cells.lock().unwrap();
            let cell_refs: Vec<_> = cells.iter().map(|cell| cell.clone()).collect();
            for cell in cells.iter_mut() {
                // Create a filtered Vec of the other cells
                let other_cells = cell_refs
                    .iter()
                    .filter_map(|other_cell| {
                        if other_cell.get_entity_id() != cell.get_entity_id() {
                            Some(other_cell.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                cell.update();
                match cell.renderer {
                    Some(renderer) => {
                        if let Some(renderer) = renderer.upgrade() {
                            renderer.into_inner().update(
                                Size::FromVolume(cell.volume()),
                                LEVEL_OF_DETAIL,
                                other_cells,
                            );
                        } else {
                            println!("No renderer present in cell!");
                        }
                    }
                    _ => {}
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

fn get_other_cells(cells: &MutexGuard<'_, Vec<(Cell, CellRenderer)>>, cell: &Cell) -> Vec<Cell> {
    let other_cells: Vec<Cell> = cells
        .iter()
        .filter(|(other, _)| other.get_entity_id() != cell.get_entity_id())
        .map(|(other_cell, _)| other_cell.clone()) // Clone the cell here
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
