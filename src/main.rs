use bevy_panorbit_camera::PanOrbitCameraPlugin;
use cgmath::Point3;
use engine::{camera::spawn_camera, simulation};

use bevy::prelude::*;
use model::{
    cell::{BiologicalCell, CellDivideEvent, CellSpawnEvent},
    tissue::{GrowingTissue, Tissue, TissueType},
};
use shared::{cell::CellInformation, math::volume_from_radius};

mod engine;
mod model;
mod shared;

pub fn spawn_light(mut commands: Commands) {
    // light
    commands.spawn((PointLight::default(), Transform::from_xyz(4.0, 8.0, 4.0)));
}

pub fn handle_spawn_cell_event(
    mut spawn_events: EventReader<CellSpawnEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in spawn_events.read() {
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(1.))),
            MeshMaterial3d(materials.add(Color::WHITE)),
            Transform::from_xyz(event.position.x, event.position.y, event.position.z),
            BiologicalCell::new(volume_from_radius(event.radius), event.tissue),
            CellInformation::<f32> {
                position: event.position,
                radius: event.radius,
            },
        ));
    }
}

pub fn spawn_cells(mut spawn_events: EventWriter<CellSpawnEvent>, mut commands: Commands) {
    let meristem = Tissue::new(TissueType::Meristem(GrowingTissue::new(Vec3::new(
        0., 1., 0.,
    ))));
    let meristem_entity = commands.spawn(meristem).id();
    spawn_events.send(CellSpawnEvent {
        position: Point3::new(0.0, 0.0, 0.0),
        radius: 1.,
        tissue: meristem_entity,
    });
    spawn_events.send(CellSpawnEvent {
        position: Point3::new(0.0, 0.0, 0.0),
        radius: 1.,
        tissue: meristem_entity,
    });
}

fn handle_cell_division(
    mut divide_events: EventReader<CellDivideEvent>,
    mut spawn_events: EventWriter<CellSpawnEvent>,
    mut cell_query: Query<(&mut BiologicalCell, &mut CellInformation<f32>)>,
    tissue_query: Query<&Tissue>,
) {
    for event in divide_events.read() {
        if let Ok((mut parent_cell, mut info)) = cell_query.get_mut(event.parent) {
            // reduce volume of parent cell
            let new_radius = info.radius / 2.;
            let new_volume = volume_from_radius(new_radius);
            parent_cell.reduce_volume(new_volume);
            info.radius = new_radius;

            let tissue_type = &tissue_query.get(parent_cell.tissue()).unwrap().tissue_type;
            match tissue_type {
                TissueType::Meristem(properties) => {
                    let mut position = info.position;
                    position.x += properties.growing_direction.x;
                    position.y += properties.growing_direction.y;
                    position.z += properties.growing_direction.z;

                    // Spawn child cell
                    spawn_events.send(CellSpawnEvent {
                        position,
                        radius: new_radius,
                        tissue: parent_cell.tissue(),
                    });
                }
                TissueType::Parenchyma => {
                    println!("Tried to divide a cell in the parenchyma!");
                }
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PanOrbitCameraPlugin)
        .add_event::<CellDivideEvent>()
        .add_event::<CellSpawnEvent>()
        .add_systems(Startup, (spawn_camera, spawn_light, spawn_cells))
        .add_systems(PreUpdate, simulation::pre_update)
        .add_systems(
            Update,
            (
                simulation::update,
                handle_spawn_cell_event,
                handle_cell_division,
            ),
        )
        .add_systems(PostUpdate, simulation::post_update)
        .run();
}
