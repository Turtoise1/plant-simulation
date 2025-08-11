use std::{
    collections::HashMap,
    fmt::Display,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

use crate::model::organ::{OrganConfig, OrganType};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct SpeciesId(&'static str);

impl Display for SpeciesId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SpeciesConfig {
    pub organs: HashMap<OrganType, OrganConfig>,
}

pub const ARABIDOPSIS_ID: SpeciesId = SpeciesId("Arabidopsis thaliana");

#[derive(Debug)]
pub struct Species {
    pub id: SpeciesId,
    pub changed_from_ui: bool,
    pub changed_from_file: Arc<Mutex<bool>>,
    pub config: SpeciesConfig,
}

impl PartialEq for Species {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.config == other.config
            && *self.changed_from_file.lock().unwrap() == *other.changed_from_file.lock().unwrap()
    }
}

impl Species {
    pub fn update_from_config_file(&mut self) {
        let mut path = "configs/species/".to_owned();
        path.push_str(self.id.to_string().as_str());
        path.push_str(".ron");
        let config: SpeciesConfig = ron::from_str(
            std::fs::read_to_string(path.as_str())
                .expect(format!("Failed to read file {:?}", path).as_str())
                .as_str(),
        )
        .expect("Failed to parse plant species");
        self.config = config;
    }

    pub fn read_from_config(id: SpeciesId) -> Self {
        let mut path = "configs/species/".to_owned();
        path.push_str(id.to_string().as_str());
        path.push_str(".ron");
        let config: SpeciesConfig = ron::from_str(
            std::fs::read_to_string(path.as_str())
                .expect(format!("Failed to read file {:?}", path).as_str())
                .as_str(),
        )
        .expect("Failed to parse plant species");
        Self {
            id,
            config,
            changed_from_file: Arc::new(Mutex::new(false)),
            changed_from_ui: false,
        }
    }
}
