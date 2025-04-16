use std::{sync::Arc, thread, time::Duration};

use cgmath::Point3;
use engine::Simulation;
use shared::cell::{Cell, EventSystem};
use winit::event_loop::{ControlFlow, EventLoop};

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

    let event_loop = EventLoop::with_user_event()
        .build()
        .expect("Event loop creation for winit failed.");

    let proxy = event_loop.create_proxy();

    App::new().add_plugins(DefaultPlugins).run();

    thread::spawn(move || loop {
        let _ = proxy.send_event(SimulationEvent::Update);
        thread::sleep(Duration::from_millis(200));
    });

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    // event_loop.set_control_flow(ControlFlow::Poll);

    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    event_loop.set_control_flow(ControlFlow::Wait);

    event_loop.run_app(&mut simulation).unwrap();
}
