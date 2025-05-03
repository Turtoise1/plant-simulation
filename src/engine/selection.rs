use bevy::{
    asset::{Assets, Handle},
    color::{
        palettes::{css::WHITE, tailwind::YELLOW_300},
        Color,
    },
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader, EventWriter},
        observer::Trigger,
        query::{With, Without},
        system::{Query, Res, ResMut},
    },
    pbr::{MeshMaterial3d, StandardMaterial},
};
use bevy_picking::events::{Click, Pointer};

use crate::model::{cell::BiologicalCell, tissue::Tissue};

use super::state::{ApplicationState, Level};

#[derive(Component)]
pub struct Selected(pub bool);

#[derive(Event)]
pub struct SelectCellEvent {
    pub target_cell: Entity,
}
#[derive(Event)]
pub struct SelectTissueEvent {
    pub target_tissue: Entity,
}

/// Returns an observer that updates the entity's material to the one specified.
pub fn selection_on_mouse_released(
    click: Trigger<Pointer<Click>>,
    state: Res<ApplicationState>,
    mut select_cell_ew: EventWriter<SelectCellEvent>,
    mut select_tissue_ew: EventWriter<SelectTissueEvent>,
    mut cell_query: Query<&BiologicalCell>,
) {
    match &*state {
        ApplicationState::Running(level) => match level {
            Level::Cells => {
                select_cell_ew.write(SelectCellEvent {
                    target_cell: click.target,
                });
            }
            Level::Tissues => {
                let bio_cell = cell_query.get_mut(click.target).unwrap();
                select_tissue_ew.write(SelectTissueEvent {
                    target_tissue: bio_cell.tissue(),
                });
            }
        },
    }
}

pub fn handle_select_cell_event(
    mut select_events: EventReader<SelectCellEvent>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cell_query: Query<(&mut MeshMaterial3d<StandardMaterial>, &mut Selected)>,
) {
    let selected_matl = materials.add(Color::from(YELLOW_300));
    let default_matl = materials.add(Color::from(WHITE));
    for select in select_events.read() {
        let (mut material, mut cell_selected) = cell_query.get_mut(select.target_cell).unwrap();
        cell_selected.0 = !cell_selected.0;
        if cell_selected.0 {
            material.0 = selected_matl.clone();
        } else {
            material.0 = default_matl.clone();
        }
    }
}

pub fn handle_select_tissue_event(
    mut select_events: EventReader<SelectTissueEvent>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cell_query: Query<
        (&mut MeshMaterial3d<StandardMaterial>, &mut Selected),
        With<BiologicalCell>,
    >,
    mut tissue_query: Query<(&Tissue, &mut Selected), Without<BiologicalCell>>,
) {
    let selected_matl = materials.add(Color::from(YELLOW_300));
    let default_matl = materials.add(Color::from(WHITE));
    for select in select_events.read() {
        let (tissue, mut tissue_selected) = tissue_query.get_mut(select.target_tissue).unwrap();
        tissue_selected.0 = !tissue_selected.0;
        for cell_entity in tissue.cell_refs.clone() {
            let (mut cell_material, mut cell_selected) = cell_query.get_mut(cell_entity).unwrap();
            cell_selected.0 = tissue_selected.0;
            if tissue_selected.0 {
                cell_material.0 = selected_matl.clone();
            } else {
                cell_material.0 = default_matl.clone();
            }
        }
    }
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
        if let Ok((mut material, selected)) = query.get_mut(trigger.target()) {
            if !selected.0 {
                material.0 = new_material.clone();
            }
        }
    }
}
