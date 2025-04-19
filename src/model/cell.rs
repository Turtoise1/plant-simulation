use bevy::ecs::component::Component;
use cgmath::{InnerSpace, Point3, Vector3};
use std::{collections::HashMap, f32::consts::E, sync::atomic::AtomicU32};

use crate::{
    model::entity::{generate_id, Entity},
    shared::{
        cell::CellInformation,
        math::{distance, mean, radius_from_volume},
    },
};

pub const SIZE_THRESHOLD: f32 = 20.;

#[derive(Clone, Copy, Debug)]
pub struct GrowthFactors {
    size_threshold: f32,
    growth_factor: f32,
    start_value: f32,
}

#[derive(Debug, Component)]
pub struct BiologicalCell {
    id: u64,
    time_lived: AtomicU32,
    growth_factors: GrowthFactors,
}

impl BiologicalCell {
    pub fn new(volume: f32) -> Self {
        let cell = BiologicalCell {
            id: generate_id(),
            time_lived: AtomicU32::new(0),
            growth_factors: GrowthFactors {
                size_threshold: SIZE_THRESHOLD,
                growth_factor: 0.0005,
                start_value: volume,
            },
        };
        cell
    }

    pub fn update<'c>(&mut self, near_cells: &HashMap<u64, &'c CellInformation<f32>>) {
        self.time_lived
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let new_volume = logistic_growth(self.growth_factors)(
            self.time_lived.load(std::sync::atomic::Ordering::Relaxed),
        );

        // self.volume = new_volume;

        // self.reposition(near_cells);
        // println!("Cell {} at {:?}", self.entity_id(), self.position());
    }
}

impl Entity for BiologicalCell {
    fn entity_id(&self) -> u64 {
        self.id
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
