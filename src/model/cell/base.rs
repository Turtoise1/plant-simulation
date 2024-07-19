use crate::model::entity::{generate_id, Entity};

pub struct Cell {
    id: u64,
}

impl Cell {
    pub fn new() -> Cell {
        Cell { id: generate_id() }
    }
}

impl Entity for Cell {
    fn get_entity_id(&self) -> u64 {
        self.id
    }
    fn update(&self) {
        println!("Cell with entity id {}", self.get_entity_id())
    }
}
