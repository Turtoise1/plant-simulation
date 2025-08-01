use bevy_egui::{EguiContextPass, EguiPlugin};
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use cgmath::Point3;
use engine::{
    camera::spawn_camera,
    gui,
    selection::{self, update_material_on, SelectCellEvent, SelectTissueEvent, Selected},
    simulation,
    state::{handle_tab_to_switch_modes, ApplicationStatePlugin},
};

use bevy::color::palettes::tailwind::*;
use bevy::prelude::*;
use model::{
    cell::{BiologicalCell, CellDivideEvent, CellSpawnEvent},
    tissue::{self, GrowingTissue, Tissue, TissueType},
};
use shared::{cell::CellInformation, math::volume_from_radius};

use crate::model::cell::CellDifferentiateEvent;

mod engine;
mod model;
mod shared;

pub fn spawn_light(mut commands: Commands) {
    // light
    commands.spawn((PointLight::default(), Transform::from_xyz(10.0, 12.0, 4.0)));
}

pub fn handle_spawn_cell_event(
    mut spawn_events: EventReader<CellSpawnEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut tissue_query: Query<&mut Tissue>,
) {
    // Set up the materials.
    let white_matl = materials.add(Color::WHITE);
    let hover_matl = materials.add(Color::from(CYAN_300));
    let selected_matl = materials.add(Color::from(YELLOW_300));

    for event in spawn_events.read() {
        let material = if event.selected {
            selected_matl.clone()
        } else {
            white_matl.clone()
        };
        let cell_entity = commands
            .spawn((
                Mesh3d(meshes.add(Sphere::new(1.))),
                MeshMaterial3d(material),
                Transform::from_xyz(event.position.x, event.position.y, event.position.z),
                BiologicalCell::new(volume_from_radius(event.radius), event.tissue),
                CellInformation::<f32> {
                    position: event.position,
                    radius: event.radius,
                },
                Selected(event.selected),
            ))
            .observe(update_material_on::<Pointer<Over>>(hover_matl.clone()))
            .observe(update_material_on::<Pointer<Out>>(white_matl.clone()))
            .observe(selection::selection_on_mouse_released)
            .id();
        let mut tissue = tissue_query.get_mut(event.tissue).unwrap();
        tissue.cell_refs.push(cell_entity);
    }
}

pub fn spawn_cells(mut spawn_events: EventWriter<CellSpawnEvent>, mut commands: Commands) {
    let meristem = Tissue::new(TissueType::Meristem(GrowingTissue::new(Vec3::new(
        0., 1., 0.,
    ))));
    let meristem_entity = commands.spawn((meristem, Selected(false))).id();
    spawn_events.write(CellSpawnEvent {
        position: Point3::new(0.0, 0.0, 0.0),
        radius: 1.,
        tissue: meristem_entity,
        selected: false,
    });
    spawn_events.write(CellSpawnEvent {
        position: Point3::new(0.0, 0.0, 0.0),
        radius: 1.,
        tissue: meristem_entity,
        selected: false,
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
                simulation::handle_cell_division,
                simulation::handle_cell_differentiation,
                handle_spawn_cell_event,
                handle_tab_to_switch_modes,
                selection::handle_select_cell_event,
                selection::handle_select_tissue_event,
            ),
        )
        .add_systems(
            PostUpdate,
            (simulation::post_update, tissue::update_central_cells),
        )
        .add_systems(EguiContextPass, gui::show_tissues_or_cells)
        .run();
}
