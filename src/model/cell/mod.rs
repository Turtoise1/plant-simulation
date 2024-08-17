use std::f32::consts::E;

use crate::model::entity::{generate_id, Entity};

pub struct Cell {
    id: u64,
    time_lived: u32,
    growth: Box<dyn Fn(u32) -> f32>,
    position: [f32; 3],
    volume: f32,
}

impl Cell {
    pub fn new(position: [f32; 3], volume: f32) -> Cell {
        Cell {
            id: generate_id(),
            time_lived: 0,
            growth: Box::new(logistic_growth(20., 0.001, volume)),
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
        self.time_lived = self.time_lived + 1;
        self.volume = self.growth.as_ref()(self.time_lived);

        println!("Cell {} has volume {}", self.get_entity_id(), self.volume());
    }
}

fn logistic_growth(threshold: f32, growth_factor: f32, start_value: f32) -> impl Fn(u32) -> f32 {
    // f'(t)=k*f(t)*(G-f(t))
    // => f(t)=1/(1+e^(-k*G*t)*(G/f(0)-1))
    return move |t: u32| {
        threshold
            / (1.
                + f32::powf(E, -growth_factor * threshold * t as f32)
                    * (threshold / start_value - 1.))
    };
}
