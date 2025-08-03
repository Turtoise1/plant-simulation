use bevy::{
    asset::Assets,
    color::Color,
    ecs::{
        entity::Entity,
        event::{Event, EventReader, EventWriter},
        system::{Commands, Query, Res, ResMut},
    },
    math::primitives::Sphere,
    pbr::{MeshMaterial3d, StandardMaterial},
    render::mesh::{Mesh, Mesh3d},
    transform::components::Transform,
};
use bevy_picking::events::{Out, Over, Pointer};
use cgmath::Point3;

use crate::{
    engine::{
        selection::{self, Selected},
        state::{ApplicationState, Level, RunningState},
    },
    model::{
        cell::BiologicalCell,
        tissue::{Tissue, TissueType},
    },
    shared::{cell::CellInformation, math::volume_from_radius},
};

#[derive(Event)]
pub struct CellDivideEvent {
    pub parent: Entity,
}

#[derive(Event)]
pub struct CellDifferentiateEvent {
    pub cell: Entity,
}

#[derive(Event)]
pub struct CellSpawnEvent {
    pub position: Point3<f32>,
    pub radius: f32,
    pub tissue: Entity,
}

pub fn handle_cell_spawn_event(
    mut spawn_events: EventReader<CellSpawnEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut tissue_query: Query<(&mut Tissue, &Selected)>,
    app_state: Res<ApplicationState>,
) {
    // Set up the materials.
    let white_matl = materials.add(Color::WHITE);
    let hover_matl = materials.add(Color::from(bevy::color::palettes::tailwind::CYAN_300));
    let selected_matl = materials.add(Color::from(bevy::color::palettes::tailwind::YELLOW_300));

    for event in spawn_events.read() {
        let (mut tissue, tissue_selected) = tissue_query.get_mut(event.tissue).unwrap();
        let material = if tissue_selected.0 {
            match &*app_state {
                ApplicationState::Running(RunningState {
                    level: Level::Tissues,
                    ..
                }) => selected_matl.clone(),
                ApplicationState::Running(RunningState {
                    level: Level::Cells,
                    ..
                }) => white_matl.clone(),
            }
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
                Selected(false),
            ))
            .observe(selection::update_material_on::<Pointer<Over>>(
                hover_matl.clone(),
                hover_matl.clone(),
            ))
            .observe(selection::update_material_on::<Pointer<Out>>(
                selected_matl.clone(),
                white_matl.clone(),
            ))
            .observe(selection::selection_on_mouse_released)
            .id();
        tissue.cell_refs.push(cell_entity);
    }
}

pub fn handle_cell_division_events(
    mut divide_events: EventReader<CellDivideEvent>,
    mut spawn_events: EventWriter<CellSpawnEvent>,
    mut cell_query: Query<(&mut BiologicalCell, &mut CellInformation<f32>, &Selected)>,
    tissue_query: Query<&Tissue>,
) {
    for event in divide_events.read() {
        if let Ok((mut parent_cell, mut info, _)) = cell_query.get_mut(event.parent) {
            // reduce volume of parent cell
            let new_radius = info.radius / 2.;
            let new_volume = volume_from_radius(new_radius);
            parent_cell.reduce_volume(new_volume);
            info.radius = new_radius;

            let tissue = tissue_query.get(parent_cell.tissue()).unwrap();
            match &tissue.tissue_type {
                TissueType::Meristem(properties) => {
                    let mut position = info.position;
                    position.x += properties.growing_direction.x;
                    position.y += properties.growing_direction.y;
                    position.z += properties.growing_direction.z;

                    // Spawn child cell
                    spawn_events.write(CellSpawnEvent {
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

pub fn handle_cell_differentiation_events(
    mut events: EventReader<CellDifferentiateEvent>,
    mut cell_query: Query<(
        &mut BiologicalCell,
        &mut CellInformation<f32>,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
    mut tissue_query: Query<(Entity, &mut Tissue, &Selected)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    app_state: Res<ApplicationState>,
) {
    let white_matl = materials.add(Color::WHITE);
    let selected_matl = materials.add(Color::from(bevy::color::palettes::tailwind::YELLOW_300));
    for event in events.read() {
        if let Ok((mut cell, _, mut cell_material)) = cell_query.get_mut(event.cell) {
            let (_, mut old_tissue, _) = tissue_query.get_mut(cell.tissue()).unwrap();
            match old_tissue.tissue_type {
                TissueType::Meristem(_) => {
                    // remove from old tissue
                    let index = old_tissue.cell_refs.iter().position(|c| *c == event.cell);
                    if let Some(index) = index {
                        old_tissue.cell_refs.remove(index);
                    } else {
                        panic!(
                            "Cell {:?} not found in tissue {:?} with cell refs {:?}",
                            event.cell,
                            cell.tissue(),
                            old_tissue.cell_refs
                        );
                    }

                    if let Some((new_tissue_entity, mut new_tissue, new_tissue_selected)) =
                        tissue_query
                            .iter_mut()
                            .find(|(_, tissue, _)| tissue.tissue_type == TissueType::Parenchyma)
                    {
                        // add to new tissue
                        new_tissue.cell_refs.push(event.cell);
                        cell.update_tissue(new_tissue_entity);
                        match &*app_state {
                            ApplicationState::Running(RunningState {
                                level: Level::Tissues,
                                ..
                            }) => {
                                let material = if new_tissue_selected.0 {
                                    selected_matl.clone()
                                } else {
                                    white_matl.clone()
                                };
                                cell_material.0 = material;
                            }
                            _ => {}
                        };
                    } else {
                        // create new tissue
                        let mut new_tissue = Tissue::new(TissueType::Parenchyma);
                        new_tissue.cell_refs.push(event.cell);
                        let tissue_entity = commands.spawn((new_tissue, Selected(false)));
                        cell.update_tissue(tissue_entity.id());
                        cell_material.0 = white_matl.clone();
                    }
                }
                _ => {
                    println!("Only meristem cells can differentiate")
                }
            }
        }
    }
}
