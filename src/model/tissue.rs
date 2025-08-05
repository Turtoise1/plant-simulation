use std::fmt::Display;

use bevy::{
    ecs::{component::Component, entity::Entity, event::EventReader, system::Query},
    math::Vec3,
};
use serde::{Deserialize, Serialize};

use crate::{
    engine::cell_events::CellSpawnEvent,
    model::{
        organ::OrganType,
        species::{PlantSpecies, SpeciesId},
    },
    shared::{cell::CellInformation, math::to_bevy_vec3},
};

#[derive(Component, Debug)]
pub struct Tissue {
    pub kind: TissueType,
    pub organ_ref: Entity,
    pub cell_refs: Vec<Entity>,
    pub config: TissueConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TissueConfig {
    pub max_cell_volume: f32,
    pub auxin_production_rate: f32,
    pub growing_config: Option<GrowingConfig>,
}

impl Default for TissueConfig {
    fn default() -> Self {
        TissueConfig {
            max_cell_volume: 20.,
            auxin_production_rate: 0.,
            growing_config: None,
        }
    }
}

impl TissueConfig {
    pub fn read_from_configs(
        species_id: SpeciesId,
        organ_type: OrganType,
        tissue_type: TissueType,
    ) -> TissueConfig {
        let mut path = "configs/species/".to_owned();
        path.push_str(species_id.to_string().as_str());
        path.push_str(".ron");
        let species: PlantSpecies = ron::from_str(
            std::fs::read_to_string(path.as_str())
                .expect(format!("Failed to read file {:?}", path).as_str())
                .as_str(),
        )
        .expect("Failed to parse plant species");
        // let serialized = ron::ser::to_string_pretty(&config, Default::default()).expect("Failed to serialize");
        if let Some(organ) = species.organs.get(&organ_type) {
            if let Some(tissue) = organ.tissues.get(&tissue_type) {
                tissue.clone()
            } else {
                panic!(
                    "Can not find tissue of type {:?} for organ of type {:?} in species with id {:?}!",
                    tissue_type, organ_type, species_id
                );
            }
        } else {
            panic!(
                "Can not find organ of type {:?} in species with id {:?}!",
                organ_type, species_id
            );
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct GrowingConfig {
    pub growing_direction: Vec3,
    pub central_cell: Option<Entity>,
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
            if let Some(growing_config) = &mut tissue.config.growing_config {
                let central_cell = cell_refs_clone.iter().max_by(|&&a, &&b| {
                    let pos_a = to_bevy_vec3(&cell_query.get(a).unwrap().position);
                    let pos_b = to_bevy_vec3(&cell_query.get(b).unwrap().position);
                    growing_config
                        .growing_direction
                        .dot(pos_a)
                        .partial_cmp(&growing_config.growing_direction.dot(pos_b))
                        .unwrap()
                });
                if let Some(central_cell) = central_cell {
                    growing_config.central_cell = Some(*central_cell);
                }
            }
        }
    }
}
