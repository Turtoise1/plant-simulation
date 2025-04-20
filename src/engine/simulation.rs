use bevy::prelude::*;

use crate::{
    model::cell::BiologicalCell,
    shared::{
        cell::{overlapping, CellInformation},
        math::to_bevy_vec3,
        overlapping_cells::OverlappingCells,
    },
};

pub fn pre_update(mut commands: Commands, query: Query<(Entity, &CellInformation<f32>)>) {
    let all_infos: Vec<(Entity, &CellInformation<f32>)> = query.iter().collect();

    for (entity, info) in &all_infos {
        let neighbors: Vec<CellInformation<f32>> = all_infos
            .iter()
            .filter(|(other_entity, other_info)| {
                *other_entity != *entity
                    && overlapping(
                        &info.position,
                        info.radius,
                        &other_info.position,
                        other_info.radius,
                    )
            })
            .map(|(_, info)| (*info).clone())
            .collect();

        // potentially empty neighbors but that's good
        commands.entity(*entity).insert(OverlappingCells(neighbors));
    }
}

pub fn update(
    mut bio_query: Query<(
        &mut Transform,
        &mut BiologicalCell,
        &mut CellInformation<f32>,
        &OverlappingCells<f32>,
    )>,
) {
    for (mut transform, mut bio, mut info, overlapping_cells) in &mut bio_query {
        let new_volume = bio.grow();
        info.update(&overlapping_cells.0, new_volume);
        transform.translation = to_bevy_vec3(&info.position);
        transform.scale.x = info.radius;
        transform.scale.y = info.radius;
        transform.scale.z = info.radius;
    }
}
