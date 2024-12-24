use std::{
    fmt::{Debug, Display, Formatter},
    sync::{Arc, Mutex},
};

use cgmath::num_traits::Num;
use tritet::{StrError, Tetgen};

use crate::model::cell::Cell;

#[derive(Debug)]
pub struct Point3<T: Num + Display> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: Num + Clone + Display> From<[T; 3]> for Point3<T> {
    fn from(value: [T; 3]) -> Self {
        Self {
            x: value[0].clone(),
            y: value[1].clone(),
            z: value[2].clone(),
        }
    }
}

impl<T: Num + Display> Display for Point3<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(x:{}, y:{}, z:{})", self.x, self.y, self.z)
    }
}

pub struct TetraederOfCells<T: Num + Copy + Display> {
    nodes: [(Point3<T>, Arc<Mutex<Cell>>); 4],
}

impl<T: Num + Copy + Display> TetraederOfCells<T> {
    pub fn new(nodes: [(Point3<T>, Arc<Mutex<Cell>>); 4]) -> Self {
        Self { nodes }
    }
    pub fn nodes(&self) -> &[(Point3<T>, Arc<Mutex<Cell>>); 4] {
        &self.nodes
    }
    pub fn points(&self) -> [&Point3<T>; 4] {
        self.nodes.each_ref().map(|n| &n.0)
    }
    pub fn cells(&self) -> [&Arc<Mutex<Cell>>; 4] {
        self.nodes.each_ref().map(|n| &n.1)
    }
    pub fn nodes_mut(&mut self) -> &mut [(Point3<T>, Arc<Mutex<Cell>>); 4] {
        &mut self.nodes
    }
    pub fn points_mut(&mut self) -> [&mut Point3<T>; 4] {
        self.nodes.each_mut().map(|n| &mut n.0)
    }
    pub fn cells_mut(&mut self) -> [&mut Arc<Mutex<Cell>>; 4] {
        self.nodes.each_mut().map(|n| &mut n.1)
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

impl<T: Num + Copy + Display> Debug for TetraederOfCells<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let points = self.points();
        write!(
            f,
            "Tetraeder [{}, {}, {}, {}]",
            points[0], points[1], points[2], points[3]
        )
    }
}

/// uses delaunay triangulation to triangulate the cells centers
/// returns the resulting tetraeders
pub fn delaunay_triangulation(
    cells: &Vec<Arc<Mutex<Cell>>>,
) -> Result<Vec<TetraederOfCells<f64>>, StrError> {
    // TODO:
    // For each edge of the tretraeders:
    //  Identify if the two connected cells do overlap on this edge
    //  Find the point where the overlap of the two cells start (away from the tetraeder)
    //  If there are more edges that have these overlaps do smart things:
    //      Find the center of the triangle that is built from the edges that have overlaps
    //      Connect the center to the point found above
    let n_points = cells.len();
    if n_points < 4 {
        return Ok(vec![]);
    }
    let mut tetgen = Tetgen::new(n_points, None, None, None)?;
    for (index, cell) in cells.iter().enumerate() {
        let pos = cell.lock().unwrap().position();
        tetgen.set_point(index, 0, pos[0] as f64, pos[1] as f64, pos[2] as f64)?;
    }

    tetgen.generate_delaunay(false)?;
    let mut tetraeders = vec![];
    for tetraeder_i in 0..tetgen.out_ncell() {
        let mut out_points = vec![];
        for m in 0..4 {
            let p = tetgen.out_cell_point(tetraeder_i, m);
            let point = Point3::<f64> {
                x: tetgen.out_point(p, 0),
                y: tetgen.out_point(p, 1),
                z: tetgen.out_point(p, 2),
            };
            out_points.push((point, Arc::clone(&cells[p])));
        }
        tetraeders.push(TetraederOfCells::new(out_points.try_into().unwrap()));
    }
    Ok(tetraeders)
}
