use std::{cell::RefCell, f32::consts::E, rc::Weak};

use crate::{
    engine::cell_renderer::CellRenderer,
    model::entity::{generate_id, Entity},
};

pub const SIZE_THRESHOLD: f32 = 20.;

#[derive(Clone, Copy)]
pub struct GrowthFactors {
    size_threshold: f32,
    growth_factor: f32,
    start_value: f32,
}

#[derive(Clone)]
pub struct Cell {
    id: u64,
    time_lived: u32,
    growth_factors: GrowthFactors,
    position: [f32; 3],
    volume: f32,
    pub renderer: Option<Weak<RefCell<CellRenderer>>>,
}

impl Cell {
    pub fn new(position: [f32; 3], volume: f32) -> Cell {
        Cell {
            id: generate_id(),
            time_lived: 0,
            growth_factors: GrowthFactors {
                size_threshold: SIZE_THRESHOLD,
                growth_factor: 0.001,
                start_value: volume,
            },
            position,
            volume,
            renderer: Option::None,
        }
    }

    pub fn position(&self) -> [f32; 3] {
        self.position
    }

    pub fn set_position(&mut self, position: [f32; 3]) {
        self.position = position;
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
        self.volume = logistic_growth(self.growth_factors)(self.time_lived);

        println!("Cell {} has volume {}", self.get_entity_id(), self.volume());
    }
}

fn logistic_growth(growth_factors: GrowthFactors) -> impl Fn(u32) -> f32 {
    // f'(t)=k*f(t)*(G-f(t))
    // => f(t)=1/(1+e^(-k*G*t)*(G/f(0)-1))
    return move |t: u32| {
        growth_factors.size_threshold
            / (1.
                + f32::powf(
                    E,
                    -growth_factors.growth_factor * growth_factors.size_threshold * t as f32,
                ) * (growth_factors.size_threshold / growth_factors.start_value - 1.))
    };
}
