use crate::model::entity::{generate_id, Entity};

pub struct Cell {
    id: u64,
    position: [f32; 3],
    volume: f32,
}

impl Cell {
    pub fn new(position: [f32; 3], volume: f32) -> Cell {
        Cell {
            id: generate_id(),
            position,
            volume,
        }
    }

    pub fn position(&self) -> [f32; 3] {
        self.position
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }
}

impl Entity for Cell {
    fn get_entity_id(&self) -> u64 {
        self.id
    }
    fn update(&mut self) {
        self.volume = self.volume() * 1.001;
        println!("Cell with entity id {}", self.get_entity_id())
    }
}
