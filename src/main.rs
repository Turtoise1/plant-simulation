use bevy_panorbit_camera::PanOrbitCameraPlugin;
use cgmath::Point3;
use engine::{camera::spawn_camera, simulation};

use bevy::prelude::*;
use model::cell::BiologicalCell;
use shared::{cell::CellInformation, math::volume_from_radius};

mod engine;
mod model;
mod shared;

pub fn spawn_light(mut commands: Commands) {
    // light
    commands.spawn((PointLight::default(), Transform::from_xyz(4.0, 8.0, 4.0)));
}

pub fn spawn_cells(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(1.))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(0.0, 0.5, 0.0),
        BiologicalCell::new(volume_from_radius(1.)),
        CellInformation::<f32> {
            position: Point3::new(0.0, 0.5, 0.0),
            radius: 1.,
        },
    ));
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(1.))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(1.0, 0.5, 0.0),
        BiologicalCell::new(volume_from_radius(1.)),
        CellInformation::<f32> {
            position: Point3::new(1.0, 0.5, 0.0),
            radius: 1.,
        },
    ));
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(1.))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(1.0, 0.5, 0.0),
        BiologicalCell::new(volume_from_radius(1.)),
        CellInformation::<f32> {
            position: Point3::new(-1.0, 0., 0.0),
            radius: 1.,
        },
    ));
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(1.))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(1.0, 0.5, 0.0),
        BiologicalCell::new(volume_from_radius(1.)),
        CellInformation::<f32> {
            position: Point3::new(-1.0, 1., 0.0),
            radius: 1.,
        },
    ));
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(1.))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(1.0, 0.5, 0.0),
        BiologicalCell::new(volume_from_radius(1.)),
        CellInformation::<f32> {
            position: Point3::new(0.0, 1., 1.0),
            radius: 1.,
        },
    ));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PanOrbitCameraPlugin)
        .add_systems(Startup, (spawn_camera, spawn_light, spawn_cells))
        .add_systems(PreUpdate, simulation::pre_update)
        .add_systems(Update, simulation::update)
        .run();
}
