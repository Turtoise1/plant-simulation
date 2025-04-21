use bevy::{
    asset::{Assets, Handle},
    color::{
        palettes::{css::WHITE, tailwind::YELLOW_300},
        Color,
    },
    ecs::{
        component::Component,
        observer::Trigger,
        query::With,
        system::{Query, ResMut},
    },
    pbr::{MeshMaterial3d, StandardMaterial},
};
use bevy_picking::events::{Click, Pointer};

use crate::model::cell::BiologicalCell;

#[derive(Component)]
pub struct Selected(pub bool);

/// Returns an observer that updates the entity's material to the one specified.
pub fn selection_on_mouse_released(
    click: Trigger<Pointer<Click>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cell_query: Query<
        (&mut MeshMaterial3d<StandardMaterial>, &mut Selected),
        With<BiologicalCell>,
    >,
) {
    let selected_matl = materials.add(Color::from(YELLOW_300));
    let default_matl = materials.add(Color::from(WHITE));
    let (mut material, mut selected) = cell_query.get_mut(click.entity()).unwrap();
    if selected.0 {
        material.0 = default_matl.clone();
    } else {
        material.0 = selected_matl.clone();
    }
    selected.0 = !selected.0;
}

/// Returns an observer that updates the entity's material to the one specified.
pub fn update_material_on<E>(
    new_material: Handle<StandardMaterial>,
) -> impl Fn(Trigger<E>, Query<(&mut MeshMaterial3d<StandardMaterial>, &Selected), With<BiologicalCell>>)
{
    // An observer closure that captures `new_material`. We do this to avoid needing to write four
    // versions of this observer, each triggered by a different event and with a different hardcoded
    // material. Instead, the event type is a generic, and the material is passed in.
    move |trigger, mut query| {
        if let Ok((mut material, selected)) = query.get_mut(trigger.entity()) {
            if !selected.0 {
                material.0 = new_material.clone();
            }
        }
    }
}
