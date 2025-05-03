use bevy::prelude::*;

use crate::{
    model::{
        cell::{BiologicalCell, CellDivideEvent, CellSpawnEvent, SIZE_THRESHOLD},
        tissue::{Tissue, TissueType},
    },
    shared::{
        cell::{overlap, CellInformation},
        math::{to_bevy_vec3, volume_from_radius},
        overlapping_cells::OverlappingCells,
    },
};

pub fn pre_update(mut commands: Commands, query: Query<(Entity, &CellInformation<f32>)>) {
    let all_infos: Vec<(Entity, &CellInformation<f32>)> = query.iter().collect();

    for (entity, info) in &all_infos {
        let neighbors: Vec<CellInformation<f32>> = all_infos
            .iter()
            .filter(|(other_entity, other_info)| {
                *other_entity != *entity && overlap(info, other_info)
            })
            .map(|(_, info)| (*info).clone())
            .collect();

        // potentially empty neighbors but that's good
        commands.entity(*entity).insert(OverlappingCells(neighbors));
    }
}

pub fn update(
    mut cell_query: Query<(
        Entity,
        &mut Transform,
        &mut BiologicalCell,
        &mut CellInformation<f32>,
        &OverlappingCells<f32>,
    )>,
    tissue_query: Query<(&Tissue,)>,
    mut events: EventWriter<CellDivideEvent>,
) {
    // grow cells
    for (_, mut transform, mut bio, mut info, overlapping_cells) in &mut cell_query {
        let new_volume = bio.update_size();
        info.update(&overlapping_cells.0, new_volume);
        transform.translation = to_bevy_vec3(&info.position);
        transform.scale.x = info.radius;
        transform.scale.y = info.radius;
        transform.scale.z = info.radius;
    }
    // divide cells if they reach a threshold
    for (entity, _, bio, info, _) in cell_query.iter() {
        let tissue_type = &tissue_query.get(bio.tissue()).unwrap().0.tissue_type;
        match tissue_type {
            TissueType::Meristem(_) => {
                if volume_from_radius(info.radius) > SIZE_THRESHOLD - 1. {
                    events.write(CellDivideEvent { parent: entity });
                }
            }
            TissueType::Parenchyma => {
                // do not divide
            }
        }
    }
}

pub fn post_update() {}

pub fn handle_cell_division(
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
