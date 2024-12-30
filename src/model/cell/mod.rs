use cgmath::{InnerSpace, Point3, Vector3};
use std::{
    collections::HashMap,
    f32::consts::E,
    sync::{atomic::AtomicU32, Arc, RwLock, RwLockReadGuard},
};

use crate::{
    engine::cell_renderer::radius_from_volume,
    model::entity::{generate_id, Entity},
    shared::{
        cell::{CellEvent, CellEventType, CellInformation, EventSystem},
        plane::{distance, mean},
    },
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
    position: Arc<RwLock<Point3<f32>>>,
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
        cell.handle_events();
        cell
    }

    pub fn position(&self) -> RwLockReadGuard<Point3<f32>> {
        self.position
            .read()
            .expect("Failed to get position from cell!")
    }

    pub fn position_clone(&self) -> Point3<f32> {
        self.position().clone()
    }

    pub fn volume(&self) -> RwLockReadGuard<f32> {
        self.volume
            .read()
            .expect("Failed to get position from cell!")
    }

    fn handle_events(&self) {
        let pos = Arc::clone(&self.position);
        let id = self.id;
        self.events.subscribe(id, move |event| {
            if event.id == id {
                match event.event_type {
                    CellEventType::UpdatePosition(new_pos) => {
                        *pos.write().unwrap() = new_pos;
                    }
                }
            };
        });
    }

    pub fn update(&self, near_cells: &HashMap<u64, CellInformation<f32>>) {
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

        self.reposition(near_cells);
        // println!("Cell {} at {:?}", self.entity_id(), self.position());
    }

    /// move self away from near cells
    fn reposition(&self, near_cells: &HashMap<u64, CellInformation<f32>>) {
        let mut positions = vec![];
        near_cells
            .values()
            .filter(|other| {
                distance(&self.position_clone(), &other.position)
                    < f32::max(radius_from_volume(&self.volume()), other.radius)
            })
            .for_each(|near| {
                positions.push(self.get_point_away_from(near));
            });
        if positions.len() > 0 {
            let event = CellEvent {
                id: self.entity_id(),
                event_type: CellEventType::UpdatePosition(mean(&positions)),
            };
            self.events.notify(Arc::new(event));
        }
    }

    /// finds a point away from the other cell and returns it
    fn get_point_away_from(&self, from: &CellInformation<f32>) -> Point3<f32> {
        let p1 = &self.position_clone();
        let p2 = &from.position;
        let r1 = radius_from_volume(&self.volume());
        let r2 = from.radius;

        let direction = Vector3::<f32> {
            x: p1.x - p2.x,
            y: p1.y - p2.y,
            z: p1.z - p2.z,
        }
        .normalize();

        let current_dist = distance(p1, p2);
        let to_dist = f32::max(r1, r2);
        let dist = to_dist - current_dist;
        Point3::<f32> {
            x: p1.x + direction.x * dist,
            y: p1.y + direction.y * dist,
            z: p1.z + direction.z * dist,
        }
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
