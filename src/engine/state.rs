use bevy::{app::Plugin, ecs::system::Resource};

pub struct ApplicationStatePlugin {}
impl Plugin for ApplicationStatePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_resource::<ApplicationState>();
    }
}

#[derive(Resource)]
pub enum ApplicationState {
    Running,
}

impl Default for ApplicationState {
    fn default() -> Self {
        ApplicationState::Running
    }
}

impl ApplicationState {
    pub fn new() -> Self {
        ApplicationState::default()
    }
}
