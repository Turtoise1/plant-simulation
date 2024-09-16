use tritet::{StrError, Tetgen, Trigen};

use crate::model::cell::{Cell, SIZE_THRESHOLD};

use super::vertex::Vertex;
use std::{
    f32::consts::PI,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct CellRenderer {
    radius: f32,
    position: [f32; 3],
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub cell: Arc<Mutex<Cell>>,
}

pub enum Size {
    FromRadius(f32),
    FromVolume(f32),
}

impl CellRenderer {
    pub fn new(cell: Arc<Mutex<Cell>>, position: [f32; 3]) -> Self {
        let cell = CellRenderer {
            radius: 0., // temporary, gets overriden in first update
            position,
            vertices: Vec::new(),
            indices: Vec::new(),
            cell,
        };

        cell
    }

    pub fn position(&self) -> [f32; 3] {
        self.position
    }

    pub fn set_position(&mut self, new_position: [f32; 3]) {
        self.position = new_position;
    }

    pub fn update(&mut self, new_size: Size, lod: u16, other_cells: Vec<Arc<Mutex<Cell>>>) {
        let triangle_centers = self.delaunay_triangulation(&other_cells).unwrap();

        let near_cells: Vec<Arc<Mutex<Cell>>> = other_cells
            .into_iter()
            .filter(|cell| near(cell, self.position))
            .collect();
        //self.reposition(&near_cells);
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

    /// move self and near cells away from each other depending on their volumes
    /// the goal is to achieve that no cells radius overlaps the core position of another cell, only outer parts
    fn reposition(&mut self, near_cells: &Vec<Arc<Mutex<Cell>>>) {
        for other_cell in near_cells {
            let other_radius;
            let other_pos;
            {
                let other_cell = other_cell.lock().unwrap();
                other_radius = radius_from_volume(other_cell.volume());
                other_pos = other_cell.position();
            }
            let min_dist = f32::max(self.radius, other_radius) // this value would make sense
                + f32::min(1., f32::min(self.radius, other_radius)); // but let's add this value because the algorithm does not work otherwise
            if distance(self.position, other_pos) < min_dist {
                self.push_away(other_cell, min_dist);
            }
        }
    }

    /// move self and other_cell away from each other depending on their volume
    /// returns the new position of the cell which has not been set yet.
    fn push_away(&mut self, other_cell: &Arc<Mutex<Cell>>, to_dist: f32) -> [f32; 3] {
        let p1 = self.position;
        let w1 = self.radius;
        let p2;
        let w2;
        {
            let other_cell = other_cell.lock().unwrap();
            p2 = other_cell.position().clone();
            w2 = radius_from_volume(other_cell.volume().clone());
        }
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

        // apply new position to the other cell
        {
            let mut other_cell = other_cell.lock().unwrap();
            let mut position = other_cell.position();
            position
                .iter_mut()
                .zip(direction_normalized.iter())
                .for_each(|(p2_comp, &dir_comp)| {
                    *p2_comp -= dir_comp * displacement * factor2;
                });
            other_cell.set_position(position);
        }

        // cannot apply new position to this cell because it is locked. So instead we return the new position
        let mut position;
        {
            position = self.position;
            position
                .iter_mut()
                .zip(direction_normalized.iter())
                .for_each(|(p1_comp, &dir_comp)| {
                    *p1_comp -= dir_comp * displacement * factor1;
                });
            self.set_position(position);
        }
        position
    }

    /// if there are other cells nearby, this method moves the vertex position towards the cell middle
    /// such that there are no overlapping parts between the near cells
    fn get_rid_of_intersections(
        &self,
        vertex: Vertex,
        near_cells: &Vec<Arc<Mutex<Cell>>>,
    ) -> Vertex {
        let intersections: Vec<&Arc<Mutex<Cell>>> = near_cells
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
            let middle = midpoint(lower_bound, vertex.position);
            // TODO : middle? that doesnt even work in the easiest case stupid ass
            // TODO : simply taking the middle does not work if two cells are nearer together than at least of one their radiuses...
            Vertex {
                position: middle,
                color: vertex.color,
            }
        }
    }

    /// uses delaunay triangulation to triangulate the cells centers and returns the centroids of the tetraeders
    fn delaunay_triangulation(
        &self,
        other_cells: &Vec<Arc<Mutex<Cell>>>,
    ) -> Result<Vec<[f64; 3]>, StrError> {
        let n_points = other_cells.len() + 1;
        if n_points < 4 {
            return Ok(vec![]);
        }
        let mut tetgen = Tetgen::new(n_points, None, None, None)?;
        for (index, cell) in other_cells.iter().enumerate() {
            let pos = cell.lock().unwrap().position();
            tetgen.set_point(index, 0, pos[0] as f64, pos[1] as f64, pos[2] as f64)?;
        }
        tetgen.set_point(
            n_points - 1,
            0,
            self.position[0] as f64,
            self.position[1] as f64,
            self.position[2] as f64,
        )?;

        tetgen.generate_delaunay(false)?;
        let mut centers = vec![];
        for tetraeder_i in 0..tetgen.out_ncell() {
            let mut out_points = vec![];
            for m in 0..4 {
                let p = tetgen.out_cell_point(tetraeder_i, m);
                let point: [f64; 3] = [
                    tetgen.out_point(p, 0),
                    tetgen.out_point(p, 1),
                    tetgen.out_point(p, 2),
                ];
                out_points.push(point);
            }
            let x = out_points[0][0] + out_points[1][0] + out_points[2][0] + out_points[3][0];
            let x = x / 4.;
            let y = out_points[0][1] + out_points[1][1] + out_points[2][1] + out_points[3][1];
            let y = y / 4.;
            let z = out_points[0][2] + out_points[1][2] + out_points[2][2] + out_points[3][2];
            let z = z / 4.;
            centers.push([x, y, z]);
        }
        Ok(centers)
    }
}

pub fn radius_from_volume(volume: f32) -> f32 {
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

fn near(cell: &Arc<Mutex<Cell>>, position: [f32; 3]) -> bool {
    return distance(cell.lock().unwrap().position(), position) <= SIZE_THRESHOLD * 2.;
}

enum Point {
    FromVertex(Vertex),
    FromF32([f32; 3]),
}
fn in_range(point: Point, cell: &Arc<Mutex<Cell>>) -> bool {
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
        cell_radius = radius_from_volume(cell.volume().clone());
    }
    return distance(cell_position, point_position) <= cell_radius;
}
