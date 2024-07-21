use engine::Simulation;
use model::cell::Cell;
use model::entity::Entity;
use winit::event_loop::{ControlFlow, EventLoop};

mod engine;
mod model;

fn main() {
    for _ in 0..10 {
        let c = Cell::new();
        c.update();
    }

    let mut simulation = Simulation::new();

    let event_loop = EventLoop::new().expect("Event loop creation for winit failed.");

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    //event_loop.set_control_flow(ControlFlow::Wait);

    event_loop.run_app(&mut simulation).unwrap();
}
