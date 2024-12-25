use crate::{
    model::cell::BiologicalCell,
    shared::{cell::EventSystem, point::Point3},
};

use super::{delaunay::TetraederOfCells, vertex::Vertex};
use std::{
    f32::consts::PI,
    sync::{Arc, Mutex},
};

#[derive(Clone, Debug)]
pub struct CellRenderer {
    radius: f32,
    pub position: Point3<f32>,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    events: Arc<EventSystem>,
}

pub enum Size {
    FromRadius(f32),
    FromVolume(f32),
}

impl CellRenderer {
    pub fn new(position: &Point3<f32>, volume: &f32, events: Arc<EventSystem>) -> Self {
        let renderer = CellRenderer {
            radius: radius_from_volume(volume),
            position: position.clone(),
            vertices: Vec::new(),
            indices: Vec::new(),
            events,
        };
        renderer
    }

    pub fn position(&self) -> &Point3<f32> {
        &self.position
    }

    pub fn update_position(&mut self, new_position: &Point3<f32>) {
        self.position = new_position.clone();
    }

    pub fn update(
        &mut self,
        new_size: Size,
        lod: u16,
        included_in_tetraeders: &Vec<TetraederOfCells<f64>>,
    ) {
        self.vertices = Vec::new();
        self.indices = Vec::new();

        self.radius = match new_size {
            Size::FromRadius(r) => r,
            Size::FromVolume(v) => radius_from_volume(&v),
        };

        let sector_count = lod * 2;
        let stack_count = lod;

        let sector_step = 2.0 * PI / sector_count as f32;
        let stack_step = PI / stack_count as f32;

        // let plain = self.get_intersection_plains(included_in_tetraeders);

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
                        x + self.position.x,
                        y + self.position.y,
                        z + self.position.z,
                    ],
                    color: [1., 1., 1.],
                };

                // TODO!
                // let vertex = self.get_rid_of_intersections(vertex, included_in_tetraeders);
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

    /// move self and near cells away from each other depending on their volumes
    /// the goal is to achieve that no cells radius overlaps the core position of another cell, only outer parts
    fn reposition(&mut self, near_cells: &Vec<Arc<Mutex<BiologicalCell>>>) {
        for other_cell in near_cells {
            let other_radius;
            let other_pos;
            {
                let other_cell = other_cell.lock().unwrap();
                other_radius = radius_from_volume(&other_cell.volume());
                other_pos = other_cell.position().clone();
            }
            let min_dist = f32::max(self.radius, other_radius) // this value would make sense
                + f32::min(1., f32::min(self.radius, other_radius)); // but let's add this value because the algorithm does not work otherwise
            if distance(&self.position.clone().into(), &other_pos.clone().into()) < min_dist {
                self.push_away(other_cell, min_dist);
            }
        }
    }

    /// move self and other_cell away from each other depending on their volume
    /// returns the new position of the cell which has not been set yet.
    fn push_away(&mut self, other_cell: &Arc<Mutex<BiologicalCell>>, to_dist: f32) -> [f32; 3] {
        let p1 = &self.position;
        let w1 = &self.radius;
        let p2;
        let w2;
        {
            let other_cell = other_cell.lock().unwrap();
            p2 = other_cell.position().clone();
            w2 = radius_from_volume(&other_cell.volume());
        }
        let d_target = to_dist;

        // Step 1: Compute the vector between the points
        let direction = [p2.x - p1.x, p2.y - p1.y, p2.z - p1.z];

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

        // apply new position to the other cell
        {
            let mut other_cell = other_cell.lock().unwrap();
            let mut position: [f32; 3] = other_cell.position().clone().into();
            position
                .iter_mut()
                .zip(direction_normalized.iter())
                .for_each(|(p2_comp, &dir_comp)| {
                    *p2_comp -= dir_comp * displacement * factor2;
                });
            other_cell.update_position(&position.into());
        }

        // cannot apply new position to this cell because it is locked. So instead we return the new position
        let mut position: [f32; 3] = self.position.clone().into();
        {
            position
                .iter_mut()
                .zip(direction_normalized.iter())
                .for_each(|(p1_comp, &dir_comp)| {
                    *p1_comp -= dir_comp * displacement * factor1;
                });
            self.update_position(&position.into());
        }
        position
    }

    // TODO!
    // fn get_intersection_plains(&self, included_in_tetraeders: &Vec<TetraederOfCells<f64>>) {
    //     included_in_tetraeders.iter().for_each(|tetraeder| {
    //         let nodes: Vec<(&Point3<f64>, &Arc<Mutex<Cell>>)> = tetraeder
    //             .nodes()
    //             .iter()
    //             .filter(|n| near(&n.1, self.))
    //             .map(|n| (&n.0, &n.1))
    //             .collect();
    //     });
    // }

    /// if there are other cells nearby, this method moves the vertex position towards the cell middle
    /// such that there are no overlapping parts between the near cells
    fn get_rid_of_intersections(
        &self,
        vertex: Vertex,
        near_cells: &Vec<Arc<Mutex<BiologicalCell>>>,
    ) -> Vertex {
        let intersections: Vec<&Arc<Mutex<BiologicalCell>>> = near_cells
            .iter()
            .filter(|cell| in_range(Point::FromVertex(vertex), cell))
            .collect();
        if intersections.is_empty() {
            vertex
        } else {
            // find middle between vertex and the cell that overlaps the cell the most on this angle
            let mut upper_bound = vertex.position;
            let mut lower_bound: [f32; 3] = self.position.clone().into();
            while distance(&upper_bound, &lower_bound) > 0.05 {
                let middle = midpoint(lower_bound.into(), upper_bound);
                let intersections: Vec<_> = intersections
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
            let middle = midpoint(lower_bound.into(), vertex.position);
            // TODO : middle? that doesnt even work in the easiest case stupid ass
            // TODO : simply taking the middle does not work if two cells are nearer together than at least of one their radiuses...
            Vertex {
                position: middle,
                color: vertex.color,
            }
        }
    }
}

pub fn radius_from_volume(volume: &f32) -> f32 {
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

fn distance(point1: &[f32; 3], point2: &[f32; 3]) -> f32 {
    f32::sqrt(
        (0..3) // includes 0, excludes 3
            .map(|xyz| f32::powi(point2[xyz] - point1[xyz], 2))
            .sum(),
    )
}

// fn near(cell: &Arc<Mutex<Cell>>, position: &Point3<f32>) -> bool {
//     let ref pos = cell.lock().unwrap().position();
//     return distance(pos.into(), position.into()) <= SIZE_THRESHOLD * 2.;
// }

/// returns whether the cells positions are further away from each other than sum of the radiuses
// fn intersect(cell1: &Arc<Mutex<Cell>>, cell2: &Arc<Mutex<Cell>>) -> bool {
//     let cell1 = cell1.lock().unwrap();
//     let cell2 = cell2.lock().unwrap();
//     let min_distance = radius_from_volume(&cell1.volume()) + radius_from_volume(&cell2.volume());
//     let pos1 : [f32; 3] = cell1.position().into();
//     distance(&pos1, &cell2.position()) < min_distance
// }

enum Point {
    FromVertex(Vertex),
    FromF32([f32; 3]),
}
fn in_range(point: Point, cell: &Arc<Mutex<BiologicalCell>>) -> bool {
    let point_position;
    let cell_position;
    let cell_radius;
    match point {
        Point::FromVertex(vertex) => point_position = vertex.position,
        Point::FromF32(pos) => point_position = pos,
    }
    {
        let cell = cell.lock().unwrap();
        cell_position = cell.position().clone();
        cell_radius = radius_from_volume(&cell.volume());
    }
    return distance(&cell_position.into(), &point_position) <= cell_radius;
}
