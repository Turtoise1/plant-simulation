use std::fmt::Display;

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        event::EventReader,
        system::{Query, ResMut},
    },
    math::Vec3,
};
use serde::{Deserialize, Serialize};

use crate::{
    engine::{cell_events::CellSpawnEvent, state::PlantState},
    model::organ::Organ,
    shared::{cell::CellInformation, math::point_to_bevy_vec3},
};

#[derive(Component, Debug)]
pub struct Tissue {
    pub kind: TissueType,
    pub organ_ref: Entity,
    pub cell_refs: Vec<Entity>,
    pub config: TissueConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TissueConfig {
    pub max_cell_volume: f32,
    pub auxin_production_rate: f32,
    pub growing_config: Option<GrowingConfig>,
    pub diffusion_factor: f32,
    pub active_transport_factor: f32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct GrowingConfig {
    pub growing_direction: Vec3,
    pub central_cell: Option<(Entity, CellInformation<f32>)>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub enum TissueType {
    /// A tissue of regularly dividing cells.
    Meristem,
    /// A tissue of far differentiated cells "filling up" many parts of a plant.
    Parenchyma,
}

impl Display for TissueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TissueType::Meristem => write!(f, "Meristem",),
            TissueType::Parenchyma => write!(f, "Parenchyma"),
        }
    }
}

impl Tissue {
    pub fn new(tissue_type: TissueType, config: TissueConfig, organ: Entity) -> Self {
        Tissue {
            kind: tissue_type,
            organ_ref: organ,
            cell_refs: vec![],
            config,
        }
    }
}

impl GrowingConfig {
    pub fn new(growing_direction: Vec3) -> Self {
        Self {
            growing_direction,
            central_cell: None,
        }
    }
}

pub fn update_central_cell(
    mut spawn_events: EventReader<CellSpawnEvent>,
    mut plant_state: ResMut<PlantState>,
    organ_query: Query<&Organ>,
    tissue_query: Query<&Tissue>,
    cell_query: Query<&CellInformation<f32>>,
) {
    // run only if new cells were spawned
    if !spawn_events.is_empty() {
        spawn_events.clear();
        for tissue in tissue_query.iter() {
            let organ = organ_query.get(tissue.organ_ref).unwrap();
            // get tissue from plant state to update globally
            let tissue_config = plant_state
                .tissue_config_mut(&organ.kind, &tissue.kind)
                .unwrap();

            let cell_refs_clone = tissue.cell_refs.clone();
            if let Some(growing_config) = &mut tissue_config.growing_config {
                let central_cell = cell_refs_clone.iter().max_by(|&&a, &&b| {
                    let pos_a = point_to_bevy_vec3(&cell_query.get(a).unwrap().position);
                    let pos_b = point_to_bevy_vec3(&cell_query.get(b).unwrap().position);
                    growing_config
                        .growing_direction
                        .dot(pos_a)
                        .partial_cmp(&growing_config.growing_direction.dot(pos_b))
                        .unwrap()
                });
                if let Some(central_cell) = central_cell {
                    growing_config.central_cell = Some((
                        *central_cell,
                        cell_query.get(*central_cell).unwrap().clone(),
                    ));
                    plant_state.set_changed_from_ui(true);
                }
            }
        }
    }
}
