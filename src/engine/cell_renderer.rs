use super::{
    delaunay::{CellInformation, TetGenResult},
    vertex::Vertex,
};
use crate::{
    model::cell::BiologicalCell,
    shared::{
        cell::EventSystem,
        plane::{point_vs_plane, signed_distance, Classification, Plane},
    },
};
use cgmath::{InnerSpace, Point3, Vector3};
use rand::random;
use std::{
    collections::HashMap,
    f32::consts::PI,
    sync::{Arc, Mutex},
};

#[derive(Clone, Debug)]
pub struct CellRenderer {
    pub radius: f32,
    pub position: Point3<f32>,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub cell_id: u64,
    events: Arc<EventSystem>,
}

pub enum Size {
    FromRadius(f32),
    FromVolume(f32),
}

impl CellRenderer {
    pub fn new(position: &Point3<f32>, volume: &f32, id: u64, events: Arc<EventSystem>) -> Self {
        let renderer = CellRenderer {
            radius: radius_from_volume(volume),
            position: position.clone(),
            vertices: Vec::new(),
            indices: Vec::new(),
            cell_id: id,
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

    pub fn update(&mut self, new_size: Size, lod: u16, included_in_tetraeders: &TetGenResult<f32>) {
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

        let planes = self.create_intersection_planes_from_tet_gen_result(included_in_tetraeders);

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

                let vertex = self.get_rid_of_intersections(vertex, &planes);
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

    fn create_intersection_planes_from_tet_gen_result(
        &self,
        tet_gen_result: &TetGenResult<f32>,
    ) -> HashMap<u64, Plane<f32>> {
        let mut planes = HashMap::<u64, Plane<f32>>::new();
        match tet_gen_result {
            TetGenResult::Success(tetraeders) => {
                tetraeders.iter().for_each(|tetraeder| {
                    let p_tet =
                        self.create_intersection_planes_from_cell_information(tetraeder.nodes());
                    p_tet.iter().for_each(|p| {
                        planes.insert(p.0, p.1.clone());
                    });
                });
            }
            TetGenResult::TooFewCells(cells) => {
                let p = self.create_intersection_planes_from_cell_information(cells);
                p.iter().for_each(|p| {
                    planes.insert(p.0, p.1.clone());
                });
            }
        }
        planes
    }

    /// returns a vector that contains for each of the input cells that intersects with self
    /// - the id of the other cell
    /// - a plane dividing self and the other cell in a good way
    fn create_intersection_planes_from_cell_information(
        &self,
        cells: &[CellInformation<f32>],
    ) -> Vec<(u64, Plane<f32>)> {
        let mut planes = vec![];
        let intersecting_cells: Vec<&CellInformation<f32>> = cells
            .iter()
            .filter(|c| c.id != self.cell_id)
            .filter(|c| intersect(&self, &c))
            .collect();
        intersecting_cells.iter().for_each(|cell| {
            planes.push((cell.id, self.get_intersection_plane_one_other(cell)));
        });
        planes
    }

    /// expects that this cell and the given other cell intersect
    /// returns a plain that separates both cells fairly
    fn get_intersection_plane_one_other(&self, other: &CellInformation<f32>) -> Plane<f32> {
        let p1 = self.position();
        let self_as_ci = CellInformation {
            id: self.cell_id,
            position: self.position,
            radius: self.radius,
        };
        let middle = between_depending_on_radius(&self_as_ci, other);
        let p2 = other.position;
        let v_cell_to_cell = Vector3 {
            x: p2.x - p1.x,
            y: p2.y - p1.y,
            z: p2.z - p1.z,
        };
        Plane::<f32> {
            pos: Vector3 {
                x: middle.x,
                y: middle.y,
                z: middle.z,
            },
            normal: v_cell_to_cell.normalize(),
        }
    }

    /// look at the planes that divide this cell from near neighbours
    /// if the vertex is on the same side as the center, keep it
    /// else move it to the plane
    fn get_rid_of_intersections(
        &self,
        vertex: Vertex,
        planes: &HashMap<u64, Plane<f32>>,
    ) -> Vertex {
        let mut vertex = vertex;
        planes.values().for_each(|plane| {
            let class_cell = point_vs_plane(&self.position, plane);
            let class_vertex = point_vs_plane(&vertex.position.into(), plane);
            if class_cell != class_vertex
                && class_cell != Classification::Intersects
                && class_vertex != Classification::Intersects
            {
                vertex = self.move_vertex_towards_plane(vertex, plane);
            }
        });
        vertex
    }

    fn move_vertex_towards_plane(&self, vertex: Vertex, plane: &Plane<f32>) -> Vertex {
        let vertex_pos: Point3<f32> = vertex.position.into();
        let dist = signed_distance(&vertex_pos, plane);
        Vertex {
            color: vertex.color,
            position: [
                vertex.position[0] - dist * plane.normal.x,
                vertex.position[1] - dist * plane.normal.y,
                vertex.position[2] - dist * plane.normal.z,
            ],
        }
    }
}

pub fn radius_from_volume(volume: &f32) -> f32 {
    // r = ((3V)/(4PI))^(1/3)
    f32::powf((3. * volume) / (4. * PI), 1. / 3.)
}

fn between_depending_on_radius(
    cell1: &CellInformation<f32>,
    cell2: &CellInformation<f32>,
) -> Point3<f32> {
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
    let factor = (radius_capped2 - overlap / 2.) / dist;
    Point3::<f32> {
        x: cell2.position.x + factor * (cell1.position.x - cell2.position.x),
        y: cell2.position.y + factor * (cell1.position.y - cell2.position.y),
        z: cell2.position.z + factor * (cell1.position.z - cell2.position.z),
    }
}

fn orthogonal_normal(vec: &Vector3<f32>) -> Vector3<f32> {
    let tangent = random_tangent(vec);
    let orthogonal = vec.cross(tangent);
    orthogonal.normalize()
}

fn random_tangent(vec: &Vector3<f32>) -> Vector3<f32> {
    Vector3 {
        x: vec.x,
        y: random::<f32>() * 360.,
        z: vec.z,
    }
}

fn distance(point1: &Point3<f32>, point2: &Point3<f32>) -> f32 {
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
fn intersect(cell1: &CellRenderer, cell2: &CellInformation<f32>) -> bool {
    let min_distance = cell1.radius + cell2.radius;
    distance(&cell1.position, &cell2.position) < min_distance
}

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
    return distance(&cell_position, &point_position.into()) <= cell_radius;
}
