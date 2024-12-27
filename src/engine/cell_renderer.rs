use super::{
    delaunay::{CellInformation, TetGenResult},
    vertex::Vertex,
};
use crate::{
    model::cell::BiologicalCell,
    shared::{
        cell::{CellEvent, CellEventType, EventSystem},
        plane::{point_vs_plane, signed_distance, Classification, Plane},
    },
};
use cgmath::{InnerSpace, Point3, Vector3};
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

    pub fn update(&mut self, new_size: Size, lod: u16, tet_gen_result: &TetGenResult<f32>) {
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

        let near_cells = self.get_near_cells(tet_gen_result);
        let mut planes = vec![];
        near_cells.values().for_each(|c| {
            let plane = self.try_create_intersection_plane(c);
            if plane.is_some() {
                planes.push(plane.unwrap());
            }
        });
        // self.reposition(near_cells);

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
    fn reposition(&mut self, near_cells: HashMap<u64, CellInformation<f32>>) {
        near_cells.values().for_each(|other_cell| {
            let other_radius = other_cell.radius;
            let min_dist = f32::max(self.radius, other_radius) // this value would make sense
                + f32::min(1., f32::min(self.radius, other_radius)); // but let's add this value because the algorithm does not work otherwise
            if distance(&self.position, &other_cell.position) < min_dist {
                self.push_away(other_cell, min_dist);
            }
        });
    }

    /// move self and other_cell away from each other depending on their volume
    /// returns the new position of the cell which has not been set yet.
    fn push_away(&mut self, other_cell: &CellInformation<f32>, to_dist: f32) {
        let p1 = &self.position;
        let w1 = &self.radius;
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
            x: other_cell.position.x - direction_normalized.x * displacement * factor2,
            y: other_cell.position.y - direction_normalized.y * displacement * factor2,
            z: other_cell.position.z - direction_normalized.z * displacement * factor2,
        };
        let event = CellEvent {
            id: other_cell.id,
            event_type: CellEventType::UpdatePosition(position),
        };
        self.events.notify(&event);

        let position = Point3::<f32> {
            x: self.position().x - direction_normalized.x * displacement * factor1,
            y: self.position().y - direction_normalized.y * displacement * factor1,
            z: self.position().z - direction_normalized.z * displacement * factor1,
        };
        let event = CellEvent {
            id: self.cell_id,
            event_type: CellEventType::UpdatePosition(position),
        };
        self.events.notify(&event);
    }

    /// If the tetraeder generation was successful:
    /// For each tetraeder where self is included, all other cells are returned.
    ///
    /// If no tetraeders have been generated:
    /// All cells where the distance is smaller than the sum of their radi are returned.
    fn get_near_cells(
        &self,
        tet_gen_result: &TetGenResult<f32>,
    ) -> HashMap<u64, CellInformation<f32>> {
        let mut near_cells = HashMap::<u64, CellInformation<f32>>::new();
        match tet_gen_result {
            TetGenResult::Success(tetraeders) => {
                tetraeders
                    .iter()
                    .filter(|t| t.nodes().iter().any(|c| c.id == self.cell_id))
                    .for_each(|tetraeder| {
                        tetraeder.nodes().iter().for_each(|c| {
                            near_cells.insert(c.id, c.clone());
                        });
                    });
            }
            TetGenResult::TooFewCells(result_cells) => {
                result_cells
                    .iter()
                    .filter(|c| near(&self, &c))
                    .for_each(|c| {
                        near_cells.insert(c.id, c.clone());
                    });
            }
        }
        near_cells
    }

    /// Returns None if other is the same as self or the two cells do not overlap
    /// Returns Some(plane) otherwise where plane divides the two cellsin a good way
    fn try_create_intersection_plane(&self, other: &CellInformation<f32>) -> Option<Plane<f32>> {
        if other.id == self.cell_id {
            return None;
        }
        if !intersect(&self, other) {
            return None;
        }
        Some(self.create_intersection_plane(other))
    }

    /// expects that this cell and the given other cell intersect
    /// returns a plain that separates both cells fairly
    fn create_intersection_plane(&self, other: &CellInformation<f32>) -> Plane<f32> {
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
    fn get_rid_of_intersections(&self, vertex: Vertex, planes: &Vec<Plane<f32>>) -> Vertex {
        let mut vertex = vertex;
        planes.iter().for_each(|plane| {
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

/// expects two overlapping cells
/// calculates the overlap in a straight line between the cells centers
/// returns the point in the middle of the overlap
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

fn distance(point1: &Point3<f32>, point2: &Point3<f32>) -> f32 {
    f32::sqrt(
        (0..3) // includes 0, excludes 3
            .map(|xyz| f32::powi(point2[xyz] - point1[xyz], 2))
            .sum(),
    )
}

fn near(cell1: &CellRenderer, cell2: &CellInformation<f32>) -> bool {
    let dist = distance(cell1.position(), &cell2.position);
    dist < cell1.radius + cell2.radius
}

/// returns whether the cells positions are further away from each other than sum of the radiuses
fn intersect(cell1: &CellRenderer, cell2: &CellInformation<f32>) -> bool {
    let min_distance = cell1.radius + cell2.radius;
    distance(&cell1.position, &cell2.position) < min_distance
}
