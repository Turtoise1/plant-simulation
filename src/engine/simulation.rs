use bevy::prelude::*;

use crate::{
    model::{
        cell::{BiologicalCell, CellDivideEvent, SIZE_THRESHOLD},
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
                    events.send(CellDivideEvent { parent: entity });
                }
            }
            TissueType::Parenchyma => {
                // do not divide
            }
        }
    }
}

pub fn post_update() {}
