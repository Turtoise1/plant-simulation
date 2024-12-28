use cgmath::Point3;
use std::{
    f32::consts::E,
    sync::{atomic::AtomicU32, Arc, RwLock, RwLockReadGuard},
};

use crate::{
    model::entity::{generate_id, Entity},
    shared::cell::{CellEvent, EventSystem},
};

pub const SIZE_THRESHOLD: f32 = 20.;

#[derive(Clone, Copy, Debug)]
pub struct GrowthFactors {
    size_threshold: f32,
    growth_factor: f32,
    start_value: f32,
}

#[derive(Debug)]
pub struct BiologicalCell {
    id: u64,
    time_lived: AtomicU32,
    growth_factors: GrowthFactors,
    pub position: Arc<RwLock<Point3<f32>>>,
    volume: Arc<RwLock<f32>>,
    events: Arc<EventSystem>,
}

impl BiologicalCell {
    pub fn new(position: &Point3<f32>, volume: f32, events: Arc<EventSystem>) -> Self {
        let cell = BiologicalCell {
            id: generate_id(),
            time_lived: AtomicU32::new(0),
            growth_factors: GrowthFactors {
                size_threshold: SIZE_THRESHOLD,
                growth_factor: 0.0005,
                start_value: volume,
            },
            position: Arc::new(RwLock::new(position.clone())),
            volume: Arc::new(RwLock::new(volume)),
            events,
        };
        cell
    }

    pub fn position(&self) -> RwLockReadGuard<Point3<f32>> {
        self.position
            .read()
            .expect("Failed to get position from cell!")
    }

    pub fn volume(&self) -> RwLockReadGuard<f32> {
        self.volume
            .read()
            .expect("Failed to get position from cell!")
    }
}

impl Entity for BiologicalCell {
    fn entity_id(&self) -> u64 {
        self.id
    }
    fn update(&mut self) {
        self.time_lived
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let new_volume = logistic_growth(self.growth_factors)(
            self.time_lived.load(std::sync::atomic::Ordering::Relaxed),
        );
        {
            let mut vol_mut = self
                .volume
                .write()
                .expect("Failed to update volume of cell!");
            *vol_mut = new_volume;
        }

        // println!("Cell {} at {:?}", self.entity_id(), self.position());
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
