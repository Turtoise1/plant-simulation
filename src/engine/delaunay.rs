use crate::{model::entity::Entity, shared::cell::Cell};
use cgmath::{BaseFloat, Point3};
use std::fmt::Debug;
use tritet::{StrError, Tetgen};

use super::cell_renderer::{radius_from_volume, CellRenderer};

#[derive(Clone, Debug)]
pub struct CellInformation<T: BaseFloat> {
    pub id: u64,
    pub position: Point3<T>,
    pub radius: T,
}

impl From<CellRenderer> for CellInformation<f32> {
    fn from(value: CellRenderer) -> Self {
        Self {
            id: value.cell_id,
            position: value.position().clone(),
            radius: value.radius,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TetraederOfCells<T: BaseFloat> {
    nodes: [CellInformation<T>; 4],
}

pub enum TetGenResult<T: BaseFloat> {
    Success(Vec<TetraederOfCells<T>>),
    TooFewCells(Vec<CellInformation<T>>),
}

impl<T: BaseFloat> TetraederOfCells<T> {
    pub fn new(nodes: [CellInformation<T>; 4]) -> Self {
        Self { nodes }
    }
    pub fn nodes(&self) -> &[CellInformation<T>; 4] {
        &self.nodes
    }
    pub fn points(&self) -> [&Point3<T>; 4] {
        self.nodes.each_ref().map(|n| &n.position)
    }
    pub fn nodes_mut(&mut self) -> &mut [CellInformation<T>; 4] {
        &mut self.nodes
    }
    pub fn points_mut(&mut self) -> [&mut Point3<T>; 4] {
        self.nodes.each_mut().map(|n| &mut n.position)
    }
    pub fn center(&self) -> Point3<T> {
        let mut x = T::zero();
        let mut y = T::zero();
        let mut z = T::zero();
        self.points().iter().for_each(|point| {
            x = x + point.x;
            y = y + point.y;
            z = z + point.z;
        });
        [x, y, z].into()
    }
}

/// uses delaunay triangulation to triangulate the cells centers
/// returns the resulting tetraeders
pub fn delaunay_triangulation(cells: &Vec<Cell>) -> Result<TetGenResult<f32>, StrError> {
    let n_points = cells.len();
    if n_points < 4 {
        let information: Vec<CellInformation<f32>> = cells
            .iter()
            .map(|c| {
                let renderer = c.renderer.read().unwrap();
                CellInformation::<f32> {
                    id: renderer.cell_id,
                    position: renderer.position().clone(),
                    radius: renderer.radius,
                }
            })
            .collect();
        return Ok(TetGenResult::TooFewCells(information));
    }
    let mut tetgen = Tetgen::new(n_points, None, None, None)?;
    for (index, cell) in cells.iter().enumerate() {
        let read_guard = cell.bio.read().unwrap();
        let pos = read_guard.position();
        tetgen.set_point(index, 0, pos.x as f64, pos.y as f64, pos.z as f64)?;
        println!("index {}, pos {:?}", index, pos);
    }
    tetgen.generate_delaunay(false)?;
    println!("Test");
    let mut tetraeders = vec![];
    for tetraeder_i in 0..tetgen.out_ncell() {
        let mut out = vec![];
        for m in 0..4 {
            let p = tetgen.out_cell_point(tetraeder_i, m);
            let point = Point3::<f32> {
                x: tetgen.out_point(p, 0) as f32,
                y: tetgen.out_point(p, 1) as f32,
                z: tetgen.out_point(p, 2) as f32,
            };
            let cell = cells.get(p).unwrap();
            let bio = cell.bio.read().unwrap();
            out.push(CellInformation {
                id: bio.entity_id(),
                position: point,
                radius: radius_from_volume(&bio.volume()),
            });
        }
        tetraeders.push(TetraederOfCells::new(out.try_into().unwrap()));
    }
    Ok(TetGenResult::Success(tetraeders))
}
