use crate::model::cell::{Cell, SIZE_THRESHOLD};

use super::vertex::Vertex;
use std::{cell::RefCell, f32::consts::PI, sync::Arc};

#[derive(Clone)]
pub struct CellRenderer {
    radius: f32,
    position: [f32; 3],
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub cell: Arc<RefCell<&Cell>>,
}

pub enum Size {
    FromRadius(f32),
    FromVolume(f32),
}

impl CellRenderer {
    pub fn new(
        cell: Arc<RefCell<&Cell>>,
        size: Size,
        position: [f32; 3],
        lod: u16,
        other_cells: Vec<Cell>,
    ) -> Self {
        let mut cell = CellRenderer {
            radius: 0., // temporary, gets overriden in update
            position,
            vertices: Vec::new(),
            indices: Vec::new(),
            cell,
        };

        cell.update(size, lod, other_cells);

        cell
    }

    pub fn update(&mut self, new_size: Size, lod: u16, other_cells: Vec<Cell>) {
        let mut near_cells: Vec<Cell> = other_cells
            .into_iter()
            .filter(|cell| near(cell, self.position))
            .collect();

        self.reposition(&mut near_cells);
        let near_cells = near_cells; // does not need to be mutable anymore

        self.vertices = Vec::new();
        self.indices = Vec::new();

        self.radius = match new_size {
            Size::FromRadius(r) => r,
            Size::FromVolume(v) => radius_from_volume(v),
        };

        let sector_count = lod * 2;
        let stack_count = lod;

        let sector_step = 2.0 * PI / sector_count as f32;
        let stack_step = PI / stack_count as f32;

        for i in 0..=stack_count {
            let stack_angle = PI / 2.0 - i as f32 * stack_step;
            let xy = self.radius * stack_angle.cos();
            let z = self.radius * stack_angle.sin();

            for j in 0..=sector_count {
                let sector_angle = j as f32 * sector_step;
                let x = xy * sector_angle.cos();
                let y = xy * sector_angle.sin();

                let vertex = Vertex {
                    position: [
                        x + self.position[0],
                        y + self.position[1],
                        z + self.position[2],
                    ],
                    color: [1., 1., 1.],
                };

                let vertex = self.get_rid_of_intersections(vertex, &near_cells);

                self.vertices.push(vertex);
            }
        }

        // Generate indices
        for i in 0..stack_count {
            for j in 0..sector_count {
                let first = (i * (sector_count + 1) + j) as u16;
                let second = (first + sector_count + 1) as u16;

                self.indices.push(first);
                self.indices.push(second);
                self.indices.push(first + 1);

                self.indices.push(second);
                self.indices.push(second + 1);
                self.indices.push(first + 1);
            }
        }
    }

    // move self and near cells away from each other depending on their volumes
    // the goal is to achieve that no cells radius overlaps the core position of another cell, only outer parts.
    fn reposition(&mut self, near_cells: &mut Vec<Cell>) {
        for other_cell in near_cells {
            let min_dist = f32::max(self.radius, radius_from_volume(other_cell.volume()));
            if distance(self.position, other_cell.position()) < min_dist {
                self.push_away(other_cell, min_dist);
            }
        }
    }

    // move self and other_cell away from each other depending on their volume
    fn push_away(&mut self, other_cell: &mut Cell, to_dist: f32) {
        let p1 = self.position;
        let p2 = other_cell.position();
        let w1 = self.radius;
        let w2 = radius_from_volume(other_cell.volume());
        let d_target = to_dist;

        // Step 1: Compute the vector between the points
        let direction = [p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]];

        // Step 2: Compute the current distance
        let length = (direction[0].powi(2) + direction[1].powi(2) + direction[2].powi(2)).sqrt();

        // Normalize the direction vector
        let direction_normalized = [
            direction[0] / length,
            direction[1] / length,
            direction[2] / length,
        ];

        // Step 3: Determine how much each position should move based on the weights
        let w_total = w1 + w2;
        let factor1 = w2 / w_total; // p1's movement factor
        let factor2 = w1 / w_total; // p2's movement factor

        // Step 4: Compute the displacement needed to reach the target distance
        let displacement = d_target - length;

        // Apply the displacement to both cells
        self.position
            .iter_mut()
            .zip(direction_normalized.iter())
            .for_each(|(p1_comp, &dir_comp)| {
                *p1_comp -= dir_comp * displacement * factor1;
            });
        self.cell.borrow_mut().set_position(self.position);

        let mut position = other_cell.position();
        position
            .iter_mut()
            .zip(direction_normalized.iter())
            .for_each(|(p2_comp, &dir_comp)| {
                *p2_comp -= dir_comp * displacement * factor2;
            });
        other_cell.set_position(position);
    }

    fn get_rid_of_intersections(&self, vertex: Vertex, near_cells: &Vec<Cell>) -> Vertex {
        let intersections: Vec<&Cell> = near_cells
            .iter()
            .filter(|cell| in_range(Point::FromVertex(vertex), cell))
            .collect();
        if intersections.is_empty() {
            vertex
        } else {
            // find middle between vertex and the cell that overlaps the cell the most on this angle
            let mut upper_bound = vertex.position;
            let mut lower_bound = self.position;
            while distance(upper_bound, lower_bound) > 0.05 {
                let middle = midpoint(lower_bound, upper_bound);
                let intersections: Vec<&&Cell> = intersections
                    .iter()
                    .filter(|cell| in_range(Point::FromF32(middle), cell))
                    .collect();
                if intersections.is_empty() {
                    lower_bound = middle;
                } else {
                    upper_bound = middle;
                }
            }
            // lower_bound is now either the middle of the cell or the first point in this angle where another cell is touched
            let middle = midpoint(lower_bound, vertex.position);
            // TODO : simply taking the middle does not work if two cells are nearer together than at least of their radiuses...
            Vertex {
                position: middle,
                color: vertex.color,
            }
        }
    }
}

fn radius_from_volume(volume: f32) -> f32 {
    // r = ((3V)/(4PI))^(1/3)
    f32::powf((3. * volume) / (4. * PI), 1. / 3.)
}

fn midpoint(point1: [f32; 3], point2: [f32; 3]) -> [f32; 3] {
    [
        (point1[0] + point2[0]) / 2.0,
        (point1[1] + point2[1]) / 2.0,
        (point1[2] + point2[2]) / 2.0,
    ]
}

fn distance(point1: [f32; 3], point2: [f32; 3]) -> f32 {
    f32::sqrt(
        (0..3) // includes 0, excludes 3
            .map(|xyz| f32::powi(point2[xyz] - point1[xyz], 2))
            .sum(),
    )
}

fn near(cell: &Cell, position: [f32; 3]) -> bool {
    return distance(cell.position(), position) <= SIZE_THRESHOLD * 2.;
}

enum Point {
    FromVertex(Vertex),
    FromF32([f32; 3]),
}
fn in_range(point: Point, cell: &Cell) -> bool {
    let position;
    match point {
        Point::FromVertex(vertex) => position = vertex.position,
        Point::FromF32(pos) => position = pos,
    }
    return distance(cell.position(), position) <= radius_from_volume(cell.volume());
}
