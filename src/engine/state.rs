use std::{path::Path, sync::Arc};

use bevy::{
    app::Plugin,
    ecs::{
        entity::Entity,
        event::{Event, EventWriter},
        resource::Resource,
        system::{Query, Res, ResMut},
    },
    input::{keyboard::KeyCode, ButtonInput},
};
use notify::{RecursiveMode, Watcher};

use crate::model::{
    cell::Cell,
    organ::{Organ, OrganConfig, OrganType},
    species::{Species, EQUISETUM_ID},
    tissue::{Tissue, TissueConfig, TissueType},
};

pub struct ApplicationStatePlugin;
impl Plugin for ApplicationStatePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_resource::<ApplicationState>();
        app.init_resource::<SimulationTime>();
        app.init_resource::<PlantState>();
    }
}

#[derive(Event)]
pub struct ApplicationStateChanged {}

#[derive(Resource, PartialEq)]
pub enum ApplicationState {
    Running(RunningState),
}

#[derive(PartialEq)]
pub struct RunningState {
    pub level: Level,
    pub speed: f32,
}

#[derive(Resource)]
pub struct PlantState(Species);

impl PlantState {
    pub fn dirty(&self) -> bool {
        *self.0.dirty.lock().unwrap()
    }

    pub fn organ_config(&self, organ_type: &OrganType) -> Option<&OrganConfig> {
        self.0.config.organs.get(organ_type)
    }

    pub fn tissue_config(
        &self,
        organ_type: &OrganType,
        tissue_type: &TissueType,
    ) -> Option<&TissueConfig> {
        if let Some(organ) = self.organ_config(organ_type) {
            organ.tissues.get(tissue_type)
        } else {
            None
        }
    }
}

impl Default for PlantState {
    fn default() -> Self {
        Self(Species::read_from_config(EQUISETUM_ID))
    }
}

#[derive(Resource, PartialEq, Default)]
pub struct SimulationTime {
    pub elapsed: f32,
    pub delta_secs: f32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Level {
    Cells,
    Tissues,
}

impl Default for ApplicationState {
    fn default() -> Self {
        ApplicationState::Running(RunningState {
            level: Level::Cells,
            speed: 1.0,
        })
    }
}

impl ApplicationState {
    pub fn new() -> Self {
        ApplicationState::default()
    }
}

pub fn handle_tab_to_switch_modes(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<ApplicationState>,
    mut event_writer: EventWriter<ApplicationStateChanged>,
) {
    if keyboard_input.just_pressed(KeyCode::Tab) {
        match &mut *state {
            ApplicationState::Running(RunningState { level, .. }) => {
                *level = match level {
                    Level::Cells => Level::Tissues,
                    Level::Tissues => Level::Cells,
                };
                event_writer.write(ApplicationStateChanged {});
            }
        }
    }
}

pub fn update_plant_state(
    mut plant_state: ResMut<PlantState>,
    organ_query: Query<(Entity, &Organ)>,
    tissue_query: Query<(Entity, &Tissue)>,
    cell_query: Query<&mut Cell>,
) {
    if plant_state.dirty() == true {
        plant_state.0.update_from_config();
        for mut cell in cell_query {
            for (tissue_entity, tissue) in tissue_query {
                if cell.tissue() == tissue_entity {
                    for (organ_entity, organ) in organ_query {
                        if tissue.organ_ref == organ_entity {
                            let config = plant_state.tissue_config(&organ.kind, &tissue.kind);
                            if let Some(config) = config {
                                cell.update_tissue_config(config.clone());
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn setup_config_watcher(plant_state: Res<PlantState>) {
    let dirty_clone = Arc::clone(&plant_state.0.dirty);
    let mut watcher =
        notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                if event.kind.is_modify() {
                    *dirty_clone.lock().unwrap() = true;
                }
            }
        })
        .unwrap();

    watcher
        .watch(Path::new("configs/species/"), RecursiveMode::Recursive)
        .unwrap();

    // Store watcher somewhere persistent
    std::mem::forget(watcher); // Or store as resource
}
