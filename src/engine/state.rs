use bevy::{
    app::Plugin,
    ecs::{
        resource::Resource,
        system::{Res, ResMut},
    },
    input::{keyboard::KeyCode, ButtonInput},
};

pub struct ApplicationStatePlugin;
impl Plugin for ApplicationStatePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_resource::<ApplicationState>();
    }
}

#[derive(Resource, PartialEq, Eq)]
pub enum ApplicationState {
    Running(Level),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Level {
    Cells,
    Tissues,
}

impl Default for ApplicationState {
    fn default() -> Self {
        ApplicationState::Running(Level::Cells)
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
) {
    if keyboard_input.just_pressed(KeyCode::Tab) {
        match &mut *state {
            ApplicationState::Running(current_level) => {
                *current_level = match current_level {
                    Level::Cells => Level::Tissues,
                    Level::Tissues => Level::Cells,
                };
                println!("Switched selection mode to {:?}", current_level);
            }
        }
    }
}
