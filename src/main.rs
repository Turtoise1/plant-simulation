use std::{thread, time::Duration};

use engine::Simulation;
use shared::{cell::Cell, point::Point3};
use winit::event_loop::{ControlFlow, EventLoop};

mod engine;
mod model;
mod shared;

enum SimulationEvent {
    Update,
}

fn main() {
    let cells = vec![
        Cell::new(
            Point3 {
                x: -1.,
                y: 0.,
                z: 0.,
            },
            10.,
        ),
        Cell::new(
            Point3 {
                x: 0.,
                y: 0.,
                z: 0.,
            },
            1.,
        ),
        Cell::new(
            Point3 {
                x: 0.,
                y: 2.,
                z: 0.,
            },
            2.,
        ),
        Cell::new(
            Point3 {
                x: -1.,
                y: 0.,
                z: 2.,
            },
            1.,
        ),
    ];

    let mut simulation = Simulation::new(cells);

    let event_loop = EventLoop::with_user_event()
        .build()
        .expect("Event loop creation for winit failed.");

    let proxy = event_loop.create_proxy();

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
