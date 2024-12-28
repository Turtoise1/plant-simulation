use super::vertex::Vertex;
use crate::shared::{
    cell::{CellEventType, CellInformation, EventSystem},
    plane::{distance, point_vs_plane, signed_distance, Classification, Plane},
};
use cgmath::{InnerSpace, Point3, Vector3};
use std::{
    collections::HashMap,
    f32::consts::PI,
    sync::{Arc, RwLock, RwLockReadGuard},
};

#[derive(Clone, Debug)]
pub struct CellRenderer {
    pub radius: f32,
    position: Arc<RwLock<Point3<f32>>>,
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
            position: Arc::new(RwLock::new(position.clone())),
            vertices: Vec::new(),
            indices: Vec::new(),
            cell_id: id,
            events,
        };
        renderer.handle_events();
        renderer
    }

    pub fn position(&self) -> RwLockReadGuard<Point3<f32>> {
        self.position.read().unwrap()
    }

    pub fn position_clone(&self) -> Point3<f32> {
        self.position.read().unwrap().clone()
    }

    fn handle_events(&self) {
        let pos = Arc::clone(&self.position);
        let id = self.cell_id;
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

    pub fn update(
        &mut self,
        new_size: Size,
        lod: u16,
        near_cells: &HashMap<u64, CellInformation<f32>>,
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

        let mut planes = vec![];
        near_cells.values().for_each(|c| {
            let plane = self.try_create_intersection_plane(c);
            if plane.is_some() {
                planes.push(plane.unwrap());
            }
        });
        let pos = self.position_clone();

        for i in 0..=stack_count {
            let stack_angle = PI / 2.0 - i as f32 * stack_step;
            let xy = self.radius * stack_angle.cos();
            let z = self.radius * stack_angle.sin();

            for j in 0..=sector_count {
                let sector_angle = j as f32 * sector_step;
                let x = xy * sector_angle.cos();
                let y = xy * sector_angle.sin();

                let vertex = Vertex {
                    position: [x + pos.x, y + pos.y, z + pos.z],
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
        let p1 = self.position_clone();
        let self_as_ci = CellInformation {
            id: self.cell_id,
            position: self.position().clone(),
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
            let class_cell = point_vs_plane(&self.position(), plane);
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

/// returns whether the cells positions are further away from each other than sum of the radiuses
fn intersect(cell1: &CellRenderer, cell2: &CellInformation<f32>) -> bool {
    let min_distance = cell1.radius + cell2.radius;
    distance(&cell1.position(), &cell2.position) < min_distance
}
