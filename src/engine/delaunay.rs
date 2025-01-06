use crate::{
    model::entity::Entity,
    shared::cell::{near, Cell, CellInformation},
};
use cgmath::{BaseFloat, Point3};
use std::{collections::HashMap, fmt::Debug};
use tritet::{StrError, Tetgen};

use super::cell_renderer::radius_from_volume;

#[derive(Clone, Debug)]
pub struct TetraederOfCells<T: BaseFloat> {
    nodes: [CellInformation<T>; 4],
}

pub enum TetGenResult<T: BaseFloat> {
    Success(Vec<TetraederOfCells<T>>),
    NoTetGenPossible(Vec<CellInformation<T>>),
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
                let position = renderer.position().clone();
                CellInformation::<f32> {
                    id: renderer.cell_id(),
                    position,
                    radius: renderer.radius_clone(),
                }
            })
            .collect();
        return Ok(TetGenResult::NoTetGenPossible(information));
    }
    let mut tetgen = Tetgen::new(n_points, None, None, None)?;
    for (index, cell) in cells.iter().enumerate() {
        let bio = cell.bio.read().unwrap();
        let pos = bio.position_clone();
        tetgen.set_point(index, 0, pos.x as f64, pos.y as f64, pos.z as f64)?;
    }
    match tetgen.generate_delaunay(false) {
        Ok(_) => {}
        Err(err) => {
            if err == "TetGen failed: points are probably coplanar" {
                println!("Warn: Coplanar cell positions. TetGen not possible.");
                let information: Vec<CellInformation<f32>> = cells
                    .iter()
                    .map(|c| {
                        let renderer = c.renderer.read().unwrap();
                        let position = renderer.position().clone();
                        CellInformation::<f32> {
                            id: renderer.cell_id(),
                            position,
                            radius: renderer.radius_clone(),
                        }
                    })
                    .collect();
                return Ok(TetGenResult::NoTetGenPossible(information));
            } else {
                return Err(err);
            }
        }
    };
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

/// If the tetraeder generation was successful:
/// For each tetraeder where self is included, all other cells are returned.
///
/// If no tetraeders have been generated:
/// All other cells where the distance is smaller than the sum of their radi are returned.
pub fn get_near_cells(
    cell: &CellInformation<f32>,
    tet_gen_result: &TetGenResult<f32>,
) -> HashMap<u64, CellInformation<f32>> {
    let mut near_cells = HashMap::<u64, CellInformation<f32>>::new();
    match tet_gen_result {
        TetGenResult::Success(tetraeders) => {
            tetraeders
                .iter()
                .filter(|t| t.nodes().iter().any(|other| other.id == cell.id))
                .for_each(|tetraeder| {
                    tetraeder
                        .nodes()
                        .iter()
                        .filter(|other| other.id != cell.id)
                        .for_each(|other| {
                            near_cells.insert(other.id, other.clone());
                        });
                });
        }
        TetGenResult::NoTetGenPossible(result_cells) => {
            result_cells
                .iter()
                .filter(|other| other.id != cell.id)
                .filter(|other| near(&cell.position, cell.radius, &other.position, other.radius))
                .for_each(|other| {
                    near_cells.insert(other.id, other.clone());
                });
        }
    }
    near_cells
}
