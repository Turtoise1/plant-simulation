use model::entity::Entity;

mod model;

fn main() {
    for _ in 0..10 {
        let c = model::cell::base::Cell::new();
        c.update();
    }
}
