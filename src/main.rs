use std::sync::Arc;

use bevy_panorbit_camera::PanOrbitCameraPlugin;
use cgmath::Point3;
use engine::{camera::spawn_camera, Simulation};
use shared::cell::{Cell, EventSystem};

use bevy::prelude::*;

mod engine;
mod model;
mod shared;

enum SimulationEvent {
    Update,
}

fn main() {
    let events = Arc::new(EventSystem::new());
    let cells = vec![
        Cell::new(
            Point3 {
                x: 0.,
                y: 0.,
                z: 0.,
            },
            1.,
            Arc::clone(&events),
        ),
        Cell::new(
            Point3 {
                x: -1.,
                y: 0.,
                z: 0.,
            },
            1.,
            Arc::clone(&events),
        ),
        Cell::new(
            Point3 {
                x: 1.,
                y: 0.,
                z: 0.,
            },
            1.,
            Arc::clone(&events),
        ),
        Cell::new(
            Point3 {
                x: 0.,
                y: -1.,
                z: 0.,
            },
            1.,
            Arc::clone(&events),
        ),
        Cell::new(
            Point3 {
                x: 0.,
                y: 1.,
                z: 0.,
            },
            1.,
            Arc::clone(&events),
        ),
        Cell::new(
            Point3 {
                x: 0.,
                y: 0.,
                z: -1.,
            },
            1.,
            Arc::clone(&events),
        ),
        Cell::new(
            Point3 {
                x: 0.,
                y: 0.,
                z: 1.,
            },
            1.,
            Arc::clone(&events),
        ),
    ];

    let mut simulation = Simulation::new(cells, events);

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PanOrbitCameraPlugin)
        .add_systems(Startup, (spawn_camera, Simulation::setup))
        .run();
}
