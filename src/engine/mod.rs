use bevy::prelude::*;
use delaunay::{delaunay_triangulation, get_near_cells};
use futures::executor::block_on;
use state::ApplicationState;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::ActiveEventLoop,
    window::Window,
};

use crate::{
    shared::cell::{Cell, EventSystem},
    SimulationEvent,
};

pub mod camera;
pub mod cell_renderer;
mod delaunay;
mod state;
mod vertex;

const LEVEL_OF_DETAIL: u16 = 20;

pub struct Simulation {
    cells: Arc<Vec<Cell>>,
    cell_events: Arc<EventSystem>,
    window: Option<Arc<Window>>,
    state: Option<ApplicationState>,
}

impl Simulation {
    pub fn new(cells: Vec<Cell>, cell_events: Arc<EventSystem>) -> Self {
        let simulation = Simulation {
            cells: Arc::new(cells),
            cell_events,
            window: None,
            state: None,
        };
        simulation
    }

    pub fn setup(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        // circular base
        commands.spawn((
            Mesh3d(meshes.add(Circle::new(4.0))),
            MeshMaterial3d(materials.add(Color::WHITE)),
            Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ));
        // cube
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
            Transform::from_xyz(0.0, 0.5, 0.0),
        ));
        // light
        commands.spawn((
            PointLight {
                shadows_enabled: true,
                ..default()
            },
            Transform::from_xyz(4.0, 8.0, 4.0),
        ));
    }

    fn render(&self, state: &ApplicationState) {}

    pub fn update(&mut self) {
        {
            let tet_gen_result;
            {
                tet_gen_result = match delaunay_triangulation(&self.cells) {
                    Ok(res) => res,
                    Err(err) => panic!("An error occured in the delaunay triangulation!\n{}", err),
                };
            }
            for cell in self.cells.iter() {
                let near_cells = get_near_cells(&cell.clone().into(), &tet_gen_result);
                {
                    let bio = cell.bio.read().unwrap();
                    bio.update(&near_cells);
                }
                {
                    let mut renderer = cell.renderer.write().unwrap();
                    renderer.update(LEVEL_OF_DETAIL, &near_cells);
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

impl ApplicationHandler<SimulationEvent> for Simulation {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(init_window(event_loop));
        self.window = Some(window.clone());
        let cells = Arc::clone(&self.cells);
        let state = block_on(ApplicationState::new(
            window,
            cells,
            Arc::clone(&self.cell_events),
        ));
        self.state = Some(state);
        println!("resumed!");
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: SimulationEvent) {
        match event {
            SimulationEvent::Update => {
                self.update();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
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
                let state = self.state.as_ref().unwrap();
                self.render(state);

                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                // self.window.as_ref().unwrap().clone().request_redraw();
            }
            WindowEvent::Resized { .. } => {
                let state = self.state.as_ref().unwrap();
                self.render(state);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let state = self.state.as_ref().unwrap();
                self.render(state);
            }
            WindowEvent::MouseInput {
                button,
                state: key_state,
                ..
            } => match button {
                MouseButton::Left => match key_state {
                    ElementState::Released => {
                        match &self.state {
                            Some(state) => {
                                // let position = state.mouse_position.as_ref().unwrap();
                                // let select_ray = state.screen_pos_2_select_ray(position);
                                // state.select_cells(select_ray);
                            }
                            None => {
                                println!("No state!")
                            }
                        }
                        // TODO: if a cell has been hit with this position, set the cell as acive and use its center as camera center.
                    }
                    _ => {}
                },
                _ => {}
            },
            WindowEvent::MouseWheel { delta, phase, .. } => {
                println!("Got mouse wheel event: {:?}, {:?}", delta, phase);
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
