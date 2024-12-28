use cgmath::Point3;
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
        plane::distance,
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

    /// move self and near cells away from each other depending on their volumes
    /// the goal is to achieve that no cells radius overlaps the core position of another cell, only outer parts
    fn reposition(&self, near_cells: &HashMap<u64, CellInformation<f32>>) {
        near_cells.values().for_each(|other_cell| {
            let other_radius = other_cell.radius;
            let min_dist = f32::max(radius_from_volume(&self.volume()), other_radius);
            if distance(&self.position(), &other_cell.position) < min_dist {
                self.push_away(other_cell, min_dist);
            }
        });
    }

    /// move self and other_cell away from each other depending on their volume
    /// returns the new position of the cell which has not been set yet.
    fn push_away(&self, other_cell: &CellInformation<f32>, to_dist: f32) {
        let p1 = &self.position();
        let w1 = radius_from_volume(&self.volume());
        let p2 = &other_cell.position;
        let w2 = &other_cell.radius;

        // Step 1: Compute the vector between the points
        let direction = Point3::<f32> {
            x: p2.x - p1.x,
            y: p2.y - p1.y,
            z: p2.z - p1.z,
        };

        // Step 2: Compute the current distance
        let length = (direction.x.powi(2) + direction.y.powi(2) + direction.z.powi(2)).sqrt();

        // Normalize the direction vector
        let direction_normalized = Point3::<f32> {
            x: direction.x / length,
            y: direction.y / length,
            z: direction.z / length,
        };

        // Step 3: Determine how much each position should move based on the weights
        let w_total = w1 + w2;
        let factor1 = w2 / w_total; // p1's movement factor
        let factor2 = w1 / w_total; // p2's movement factor

        // Step 4: Compute the displacement needed to reach the target distance
        let displacement = to_dist - length;

        let position = Point3::<f32> {
            x: p2.x + direction_normalized.x * displacement * factor2,
            y: p2.y + direction_normalized.y * displacement * factor2,
            z: p2.z + direction_normalized.z * displacement * factor2,
        };
        let event = CellEvent {
            id: other_cell.id,
            event_type: CellEventType::UpdatePosition(position),
        };
        self.events.notify(Arc::new(event));

        let position = Point3::<f32> {
            x: p1.x + direction_normalized.x * displacement * factor1,
            y: p1.y + direction_normalized.y * displacement * factor1,
            z: p1.z + direction_normalized.z * displacement * factor1,
        };
        let event = CellEvent {
            id: self.entity_id(),
            event_type: CellEventType::UpdatePosition(position),
        };
        self.events.notify(Arc::new(event));
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
