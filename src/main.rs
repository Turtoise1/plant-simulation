use engine::Simulation;
use model::cell::Cell;
use model::entity::Entity;
use winit::event_loop::EventLoop;

mod engine;
mod model;

fn main() {
    for _ in 0..10 {
        let c = Cell::new();
        c.update();
    }

    let mut simulation = Simulation::new();
    let event_loop = EventLoop::new().expect("Event loop creation for winit failed.");
    event_loop.run_app(&mut simulation).unwrap();
}
