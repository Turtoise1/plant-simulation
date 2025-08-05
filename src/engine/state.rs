use bevy::{
    app::Plugin,
    ecs::{
        event::{Event, EventWriter},
        resource::Resource,
        system::{Res, ResMut},
    },
    input::{keyboard::KeyCode, ButtonInput},
};

use crate::model::species::{SpeciesId, EQUISETUM_ID};

pub struct ApplicationStatePlugin;
impl Plugin for ApplicationStatePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_resource::<ApplicationState>();
        app.init_resource::<SimulationTime>();
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
    pub species: SpeciesId,
    pub level: Level,
    pub speed: f32,
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
            species: EQUISETUM_ID,
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
