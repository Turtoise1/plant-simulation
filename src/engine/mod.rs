use bevy::prelude::*;

use delaunay::{delaunay_triangulation, get_near_cells};

use crate::{model::cell::BiologicalCell, shared::cell::CellInformation};

pub mod camera;
mod delaunay;
mod state;
mod vertex;

pub struct Simulation {}

impl Simulation {
    pub fn new() -> Self {
        let simulation = Simulation {};
        simulation
    }

    fn update(
        info_query: Query<&CellInformation<f32>>, // read-only
        mut bio_query: Query<(&mut BiologicalCell, &CellInformation<f32>)>,
    ) {
        // 1. Gather cell info from the read-only query
        let cell_infos: Vec<&CellInformation<f32>> = info_query.iter().collect();

        // 2. Run triangulation
        let tet_gen_result = match delaunay_triangulation(&cell_infos) {
            Ok(res) => res,
            Err(err) => {
                panic!("An error occurred in the delaunay triangulation!\n{}", err)
            }
        };

        // 3. Update each cell using the mutable query
        for (mut bio, info) in &mut bio_query {
            let near_cells = get_near_cells(info, &tet_gen_result);
            bio.update(&near_cells);
        }
    }
}
