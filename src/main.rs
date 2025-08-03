use bevy_egui::{EguiContextPass, EguiPlugin};
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use cgmath::Point3;
use engine::{
    camera::spawn_camera,
    gui,
    selection::{self, SelectCellEvent, SelectTissueEvent, Selected},
    simulation,
    state::{handle_tab_to_switch_modes, ApplicationStatePlugin},
};

use bevy::prelude::*;
use model::tissue::{self, GrowingTissue, Tissue, TissueType};

use crate::engine::{
    cell_events::{self, CellDifferentiateEvent, CellDivideEvent, CellSpawnEvent},
    state::ApplicationStateChanged,
};

mod engine;
mod model;
mod shared;

pub fn spawn_light(mut commands: Commands) {
    // light
    commands.spawn((PointLight::default(), Transform::from_xyz(10.0, 12.0, 4.0)));
}

pub fn spawn_cells(mut spawn_events: EventWriter<CellSpawnEvent>, mut commands: Commands) {
    let meristem = Tissue::new(TissueType::Meristem(GrowingTissue::new(Vec3::new(
        0., 1., 0.,
    ))));
    let meristem_entity = commands.spawn((meristem, Selected(false))).id();
    spawn_events.write(CellSpawnEvent {
        position: Point3::new(0.5, 0.0, 0.0),
        radius: 0.8,
        tissue: meristem_entity,
    });
    spawn_events.write(CellSpawnEvent {
        position: Point3::new(-0.5, 0.0, 0.0),
        radius: 1.,
        tissue: meristem_entity,
    });
    spawn_events.write(CellSpawnEvent {
        position: Point3::new(0.0, 0.0, 0.5),
        radius: 0.75,
        tissue: meristem_entity,
    });
    spawn_events.write(CellSpawnEvent {
        position: Point3::new(0.0, 0.0, -0.5),
        radius: 1.,
        tissue: meristem_entity,
    });
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PanOrbitCameraPlugin,
            MeshPickingPlugin,
            EguiPlugin {
                enable_multipass_for_primary_context: true,
            },
        ))
        .add_plugins(ApplicationStatePlugin)
        .add_event::<ApplicationStateChanged>()
        .add_event::<CellDivideEvent>()
        .add_event::<CellDifferentiateEvent>()
        .add_event::<CellSpawnEvent>()
        .add_event::<SelectCellEvent>()
        .add_event::<SelectTissueEvent>()
        .add_systems(Startup, (spawn_camera, spawn_light, spawn_cells))
        .add_systems(PreUpdate, simulation::pre_update)
        .add_systems(
            Update,
            (
                simulation::update,
                cell_events::handle_cell_division_events,
                cell_events::handle_cell_differentiation_events,
                cell_events::handle_cell_spawn_event,
                handle_tab_to_switch_modes,
                selection::handle_select_cell_event,
                selection::handle_select_tissue_event,
                selection::handle_application_state_changed_event,
            ),
        )
        .add_systems(
            PostUpdate,
            (simulation::post_update, tissue::update_central_cells),
        )
        .add_systems(EguiContextPass, gui::show_gui)
        .run();
}
