use bevy::prelude::*;
use cgmath::{BaseFloat, InnerSpace, Point3, Vector3};
use std::{collections::HashMap, fmt::Debug};

use super::math::{distance, mean};

#[derive(Clone, Debug, Component)]
pub struct CellInformation<T: BaseFloat> {
    pub id: u64,
    pub position: Point3<T>,
    pub radius: T,
}

impl<T: BaseFloat + std::iter::Sum> CellInformation<T> {
    /// move self away from near cells
    fn reposition(&mut self, near_cells: &HashMap<u64, CellInformation<T>>) {
        let mut positions = vec![];
        near_cells
            .values()
            .filter(|other| {
                distance(&self.position, &other.position) < T::max(self.radius, other.radius)
            })
            .for_each(|near| {
                positions.push(self.get_point_away_from(near));
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

        let direction = Vector3::<T>::new(p1.x - p2.x, p1.y - p2.y, p1.z - p2.z).normalize();

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

/// Whether two cells with the given positions and radii overlap
pub fn overlapping<T: BaseFloat + std::iter::Sum>(
    pos1: &Point3<T>,
    radius1: T,
    pos2: &Point3<T>,
    radius2: T,
) -> bool {
    let dist = distance(pos1, pos2);
    dist < radius1 + radius2
}

/// expects two overlapping cells
/// calculates the overlap in a straight line between the cells centers
/// returns the point in the middle of the overlap
fn between_depending_on_radius<T: BaseFloat + std::iter::Sum>(
    cell1: &CellInformation<T>,
    cell2: &CellInformation<T>,
) -> Point3<T> {
    let mut radius_capped1 = cell1.radius;
    let mut radius_capped2 = cell2.radius;
    let dist = distance(&cell1.position, &cell2.position);
    if radius_capped1 > dist {
        radius_capped1 = dist;
    }
    if radius_capped2 > dist {
        radius_capped2 = dist;
    }
    // between 0 and 1
    let overlap = radius_capped1 + radius_capped2 - dist;
    let factor = (radius_capped2 - overlap / T::from(2.).unwrap()) / dist;
    Point3::<T> {
        x: cell2.position.x + factor * (cell1.position.x - cell2.position.x),
        y: cell2.position.y + factor * (cell1.position.y - cell2.position.y),
        z: cell2.position.z + factor * (cell1.position.z - cell2.position.z),
    }
}

/// returns whether the cells positions are further away from each other than sum of the radiuses
fn intersect<T: BaseFloat + std::iter::Sum>(
    cell1: &CellInformation<T>,
    cell2: &CellInformation<T>,
) -> bool {
    let min_distance = cell1.radius + cell2.radius;
    distance(&cell1.position, &cell2.position) < min_distance
}
