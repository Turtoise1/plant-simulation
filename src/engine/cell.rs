use crate::model::cell::{Cell, SIZE_THRESHOLD};

use super::vertex::Vertex;
use std::f32::consts::PI;

#[derive(Clone)]
pub struct CellRenderer {
    radius: f32,
    position: [f32; 3],
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

pub enum Size {
    FromRadius(f32),
    FromVolume(f32),
}

impl CellRenderer {
    pub fn new(size: Size, position: [f32; 3], lod: u16, other_cells: Vec<Cell>) -> Self {
        let mut cell = CellRenderer {
            radius: 0., // temporary, gets overriden in update
            position,
            vertices: Vec::new(),
            indices: Vec::new(),
        };

        cell.update_size(size, lod, other_cells);

        cell
    }

    pub fn update_size(&mut self, new_size: Size, lod: u16, other_cells: Vec<Cell>) {
        let near_cells: Vec<Cell> = other_cells
            .into_iter()
            .filter(|cell| near(cell, self.position))
            .collect();

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

    fn get_rid_of_intersections(&self, vertex: Vertex, near_cells: &Vec<Cell>) -> Vertex {
        let intersections: Vec<&Cell> = near_cells
            .iter()
            .filter(|cell| in_range(vertex, cell))
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
                    .filter(|cell| in_range(vertex, cell))
                    .collect();
                if intersections.is_empty() {
                    lower_bound = middle;
                } else {
                    upper_bound = middle;
                }
            }
            Vertex {
                position: lower_bound,
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
        (0..2)
            .map(|xyz| f32::powi(point1[xyz] - point2[xyz], 2))
            .sum(),
    )
}

fn near(cell: &Cell, position: [f32; 3]) -> bool {
    return distance(cell.position(), position) <= SIZE_THRESHOLD * 2.;
}

fn in_range(vertex: Vertex, cell: &Cell) -> bool {
    return distance(cell.position(), vertex.position) <= radius_from_volume(cell.volume());
}
