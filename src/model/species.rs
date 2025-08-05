use std::{collections::HashMap, fmt::Display};

use bevy::{
    asset::uuid::{uuid, Uuid},
    ecs::resource::Resource,
};
use serde::{de::IntoDeserializer, Deserialize, Serialize};

use crate::model::organ::{OrganConfig, OrganType};

#[derive(Resource)]
pub struct PlantConfigs {
    pub species: HashMap<SpeciesId, PlantSpecies>,
    /// When a config file has been changed, this can be set to true to update the config in the next update iteration
    pub dirty: bool,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SpeciesId(&'static str);

impl Display for SpeciesId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PlantSpecies {
    pub organs: HashMap<OrganType, OrganConfig>,
}

pub const EQUISETUM_ID: SpeciesId = SpeciesId("Equisetum arvense");
