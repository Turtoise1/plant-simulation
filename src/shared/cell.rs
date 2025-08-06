use bevy::prelude::*;
use cgmath::{BaseFloat, InnerSpace, Point3, Vector3};
use std::{f32::EPSILON, fmt::Debug};

use crate::shared::{math::add_small_random_to_one_direction, overlapping_cells::OverlappingCells};

use super::math::{distance, mean, radius_from_volume};

#[derive(Clone, Debug, Component)]
pub struct CellInformation<T: BaseFloat> {
    pub position: Point3<T>,
    pub radius: T,
}

impl<T: BaseFloat + std::iter::Sum> CellInformation<T> {
    /// Updates self.radius according to the new volume and reposition itself according to near cells
    pub fn update(&mut self, overlapping_cells: &OverlappingCells<T>, new_volume: T) {
        self.radius = radius_from_volume(new_volume);
        self.reposition(overlapping_cells);
    }

    /// move self away from near cells
    fn reposition(&mut self, overlapping_cells: &OverlappingCells<T>) {
        let mut positions = vec![];
        overlapping_cells
            .0
            .iter()
            .filter(|(_, other, _)| core_overlap(self, other))
            .for_each(|(_, core_overlap, _)| {
                positions.push(self.get_point_away_from(core_overlap));
            });
        if positions.len() > 0 {
            self.position = mean(&positions);
        }
    }

    /// finds a point away from the other cell and returns it
    fn get_point_away_from(&self, from: &CellInformation<T>) -> Point3<T> {
        let p1 = &self.position;
        let p2 = &from.position;
        let r1 = self.radius;
        let r2 = from.radius;

        let mut direction = Vector3::<T>::new(p1.x - p2.x, p1.y - p2.y, p1.z - p2.z);
        // prevent normalization with magnitude == 0
        if direction.magnitude() < T::from(EPSILON).unwrap() {
            add_small_random_to_one_direction(&mut direction);
        }
        direction = direction.normalize();

        let current_dist = distance(p1, p2);
        let to_dist = T::max(r1, r2);
        let dist = to_dist - current_dist;
        Point3::<T> {
            x: p1.x + direction.x * dist,
            y: p1.y + direction.y * dist,
            z: p1.z + direction.z * dist,
        }
    }
}

/// returns whether the cells positions are further away from each other than sum of the radiuses
pub fn overlap<T: BaseFloat + std::iter::Sum>(
    cell1: &CellInformation<T>,
    cell2: &CellInformation<T>,
) -> bool {
    let min_distance = cell1.radius + cell2.radius;
    distance(&cell1.position, &cell2.position) < min_distance
}

/// returns whether the cells positions are further away from each other than the max radius
pub fn core_overlap<T: BaseFloat + std::iter::Sum>(
    cell1: &CellInformation<T>,
    cell2: &CellInformation<T>,
) -> bool {
    let max_radius = T::max(cell1.radius, cell2.radius);
    distance(&cell1.position, &cell2.position) < max_radius
}
