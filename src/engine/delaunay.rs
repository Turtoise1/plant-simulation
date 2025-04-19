use crate::shared::cell::{overlapping, CellInformation};
use cgmath::{BaseFloat, Point3};
use std::{collections::HashMap, fmt::Debug};
use tritet::{StrError, Tetgen};

#[derive(Clone, Debug)]
pub struct TetraederOfCells<'c, T: BaseFloat> {
    nodes: [&'c CellInformation<T>; 4],
}

pub enum TetGenResult<'c, T: BaseFloat> {
    Success(Vec<TetraederOfCells<'c, T>>),
    NoTetGenPossible(Vec<&'c CellInformation<T>>),
}

impl<'c, T: BaseFloat> TetraederOfCells<'c, T> {
    pub fn new(nodes: [&'c CellInformation<T>; 4]) -> Self {
        Self { nodes }
    }
    pub fn nodes(&self) -> &[&'c CellInformation<T>; 4] {
        &self.nodes
    }
    pub fn points(&self) -> [&Point3<T>; 4] {
        self.nodes.each_ref().map(|n| &n.position)
    }
    pub fn nodes_mut(&mut self) -> &mut [&'c CellInformation<T>; 4] {
        &mut self.nodes
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
pub fn delaunay_triangulation<'c>(
    cells: &Vec<&'c CellInformation<f32>>,
) -> Result<TetGenResult<'c, f32>, StrError> {
    let n_points = cells.len();
    if n_points < 4 {
        return Ok(TetGenResult::NoTetGenPossible(cells.clone()));
    }
    let mut tetgen = Tetgen::new(n_points, None, None, None)?;
    for (index, cell) in cells.iter().enumerate() {
        let pos = cell.position;
        tetgen.set_point(index, 0, pos.x as f64, pos.y as f64, pos.z as f64)?;
    }
    match tetgen.generate_delaunay(false) {
        Ok(_) => {}
        Err(err) => {
            if err == "TetGen failed: points are probably coplanar" {
                println!("Warn: Coplanar cell positions. TetGen not possible.");
                return Ok(TetGenResult::NoTetGenPossible(cells.clone()));
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
            let cell = cells.get(p).unwrap();
            out.push(*cell);
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
pub fn get_near_cells<'c>(
    cell: &'c CellInformation<f32>,
    tet_gen_result: &TetGenResult<'c, f32>,
) -> HashMap<u64, &'c CellInformation<f32>> {
    let mut near_cells = HashMap::<u64, &'c CellInformation<f32>>::new();
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
                            near_cells.insert(other.id, other);
                        });
                });
        }
        TetGenResult::NoTetGenPossible(result_cells) => {
            result_cells
                .iter()
                .filter(|other| other.id != cell.id)
                .filter(|other| {
                    overlapping(&cell.position, cell.radius, &other.position, other.radius)
                })
                .for_each(|other| {
                    near_cells.insert(other.id, other.clone());
                });
        }
    }
    near_cells
}
