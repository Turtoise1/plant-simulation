use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventWriter},
        system::Commands,
    },
    utils::HashSet,
};
use cgmath::Point3;
use std::{f32::consts::E, sync::atomic::AtomicU32};

use super::hormone::Phytohormone;

pub const SIZE_THRESHOLD: f32 = 20.;

#[derive(Clone, Copy, Debug)]
pub struct GrowthFactors {
    size_threshold: f32,
    growth_factor: f32,
    start_value: f32,
}

#[derive(Event)]
pub struct CellDivideEvent {
    pub parent: Entity,
}

#[derive(Event)]
pub struct CellSpawnEvent {
    pub position: Point3<f32>,
    pub radius: f32,
    pub tissue: Entity,
}

#[derive(Debug, Component)]
pub struct BiologicalCell {
    time_lived: u32,
    growth_factors: GrowthFactors,
    hormones: HashSet<Phytohormone>,
    /// reference to the tissue entity this cell belongs to
    tissue: Entity,
}

impl BiologicalCell {
    pub fn new(volume: f32, tissue: Entity) -> Self {
        let cell = BiologicalCell {
            time_lived: 0,
            growth_factors: GrowthFactors {
                size_threshold: SIZE_THRESHOLD,
                growth_factor: 0.0003,
                start_value: volume,
            },
            hormones: HashSet::new(),
            tissue,
        };
        cell
    }

    pub fn update_size(&mut self) -> f32 {
        self.time_lived += 1;

        let volume = logistic_growth(self.growth_factors)(self.time_lived);

        volume
    }

    pub fn reduce_volume(&mut self, new_volume: f32) {
        self.time_lived = 0;
        self.growth_factors.start_value = new_volume;
    }

    pub fn tissue(&self) -> Entity {
        self.tissue
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
