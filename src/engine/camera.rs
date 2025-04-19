use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
        PanOrbitCamera::default(),
    ));
}
