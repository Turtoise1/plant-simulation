use std::fmt::Display;

use bevy::{
    ecs::{component::Component, entity::Entity, event::EventReader, system::Query},
    math::Vec3,
};

use crate::{
    engine::cell_events::CellSpawnEvent,
    shared::{cell::CellInformation, math::to_bevy_vec3},
};

#[derive(Component, Debug)]
pub struct Tissue {
    pub tissue_type: TissueType,
    pub cell_refs: Vec<Entity>,
}

#[derive(Debug, PartialEq)]
pub struct GrowingTissue {
    pub growing_direction: Vec3,
    pub central_cell: Option<Entity>,
}

#[derive(Debug, PartialEq)]
pub enum TissueType {
    /// A tissue of regularly dividing cells.
    Meristem(GrowingTissue),
    /// A tissue of far differentiated cells "filling up" many parts of a plant.
    Parenchyma,
}

impl Display for TissueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TissueType::Meristem(growing_tissue) => write!(
                f,
                "Meristem {{ growing_direction: {:?} }}",
                growing_tissue.growing_direction
            ),
            TissueType::Parenchyma => write!(f, "Parenchyma"),
        }
    }
}

impl Tissue {
    pub fn new(tissue_type: TissueType) -> Self {
        Tissue {
            tissue_type,
            cell_refs: vec![],
        }
    }
}

impl GrowingTissue {
    pub fn new(growing_direction: Vec3) -> Self {
        Self {
            growing_direction,
            central_cell: None,
        }
    }
}

pub fn update_central_cells(
    mut spawn_events: EventReader<CellSpawnEvent>,
    mut tissue_query: Query<&mut Tissue>,
    cell_query: Query<&CellInformation<f32>>,
) {
    // run only if new cells were spawned
    if !spawn_events.is_empty() {
        spawn_events.clear();
        for mut tissue in &mut tissue_query {
            let cell_refs_clone = tissue.cell_refs.clone();
            if let TissueType::Meristem(growing_tissue) = &mut tissue.tissue_type {
                let central_cell = cell_refs_clone.iter().max_by(|&&a, &&b| {
                    let pos_a = to_bevy_vec3(&cell_query.get(a).unwrap().position);
                    let pos_b = to_bevy_vec3(&cell_query.get(b).unwrap().position);
                    growing_tissue
                        .growing_direction
                        .dot(pos_a)
                        .partial_cmp(&growing_tissue.growing_direction.dot(pos_b))
                        .unwrap()
                });
                if let Some(central_cell) = central_cell {
                    growing_tissue.central_cell = Some(*central_cell);
                }
            }
        }
    }
}
