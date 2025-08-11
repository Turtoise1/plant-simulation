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
    species::{Species, ARABIDOPSIS_ID},
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
    pub fn name(&self) -> String {
        self.0.id.to_string()
    }

    pub fn changed_from_ui(&self) -> bool {
        self.0.changed_from_ui
    }

    pub fn set_changed_from_ui(&mut self, value: bool) {
        self.0.changed_from_ui = value;
    }

    pub fn changed_from_file(&self) -> bool {
        *self.0.changed_from_file.lock().unwrap()
    }

    pub fn set_changed_from_file(&mut self, value: bool) {
        *self.0.changed_from_file.lock().unwrap() = value;
    }

    pub fn organ_config(&self, organ_type: &OrganType) -> Option<&OrganConfig> {
        self.0.config.organs.get(organ_type)
    }

    pub fn organ_config_mut(&mut self, organ_type: &OrganType) -> Option<&mut OrganConfig> {
        self.0.config.organs.get_mut(organ_type)
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

    pub fn tissue_config_mut(
        &mut self,
        organ_type: &OrganType,
        tissue_type: &TissueType,
    ) -> Option<&mut TissueConfig> {
        if let Some(organ) = self.organ_config_mut(organ_type) {
            organ.tissues.get_mut(tissue_type)
        } else {
            None
        }
    }
}

impl Default for PlantState {
    fn default() -> Self {
        Self(Species::read_from_config(ARABIDOPSIS_ID))
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
    mut tissue_query: Query<(Entity, &mut Tissue)>,
    cell_query: Query<&mut Cell>,
) {
    if plant_state.changed_from_file() || plant_state.changed_from_ui() {
        if plant_state.changed_from_file() {
            plant_state.0.update_from_config_file();
            plant_state.set_changed_from_file(false);
        } else {
            plant_state.set_changed_from_ui(false);
        }
        for mut cell in cell_query {
            for (tissue_entity, mut tissue) in tissue_query.iter_mut() {
                if cell.tissue() == tissue_entity {
                    for (organ_entity, organ) in organ_query {
                        if tissue.organ_ref == organ_entity {
                            let config = plant_state.tissue_config(&organ.kind, &tissue.kind);
                            if let Some(config) = config {
                                tissue.config = config.clone();
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
    let dirty_clone = Arc::clone(&plant_state.0.changed_from_file);
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
