use std::collections::HashMap;

use bevy::ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::model::tissue::{TissueConfig, TissueType};

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub enum OrganType {
    Stem,
    Root,
    Leaf,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OrganConfig {
    pub tissues: HashMap<TissueType, TissueConfig>,
}

impl OrganConfig {
    pub fn new() -> Self {
        OrganConfig { tissues: [].into() }
    }
}

#[derive(Component, Debug, Clone)]
pub struct Organ {
    pub kind: OrganType,
    pub config: OrganConfig,
}

impl Organ {
    pub fn new(kind: OrganType, config: OrganConfig) -> Self {
        Organ { kind, config }
    }
}
