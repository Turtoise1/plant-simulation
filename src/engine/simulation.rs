use bevy::prelude::*;

use crate::{
    engine::{
        cell_events::{CellDifferentiateEvent, CellDivideEvent},
        state::{ApplicationState, RunningState, SimulationTime},
    },
    model::{
        cell::Cell,
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

pub fn update_simulation_time(
    time: Res<Time>,
    state: Res<ApplicationState>,
    mut sim_time: ResMut<SimulationTime>,
) {
    match &*state {
        ApplicationState::Running(RunningState { speed, .. }) => {
            sim_time.delta_secs = time.delta_secs() * speed;
            sim_time.elapsed += sim_time.delta_secs;
        }
    }
}

pub fn update(
    mut cell_query: Query<(
        Entity,
        &mut Transform,
        &mut Cell,
        &mut CellInformation<f32>,
        &OverlappingCells<f32>,
    )>,
    tissue_query: Query<&Tissue>,
    mut divide_event_writer: EventWriter<CellDivideEvent>,
    mut differentiate_event_writer: EventWriter<CellDifferentiateEvent>,
    sim_time: Res<SimulationTime>,
) {
    // grow cells
    for (_, mut transform, mut bio, mut info, overlapping_cells) in &mut cell_query {
        let new_volume = bio.update_size(sim_time.elapsed);
        bio.update_hormones(sim_time.delta_secs);
        info.update(&overlapping_cells.0, new_volume);
        transform.translation = to_bevy_vec3(&info.position);
        transform.scale.x = info.radius;
        transform.scale.y = info.radius;
        transform.scale.z = info.radius;
    }
    // divide cells if they reach a threshold
    for (entity, _, bio, info, _) in cell_query.iter() {
        if let Ok(tissue) = tissue_query.get(bio.tissue()) {
            match tissue.kind {
                TissueType::Meristem => {
                    if volume_from_radius(info.radius) > tissue.config.max_cell_volume - 1. {
                        divide_event_writer.write(CellDivideEvent { parent: entity });
                    }
                }
                TissueType::Parenchyma => {
                    // do not divide
                }
            }
        } else {
            println!("Cannot find tissue with id {:?}", bio.tissue());
        };
    }
    // handle differentiation
    for (entity, _, bio, _, _) in cell_query.iter() {
        let tissue_type = &tissue_query.get(bio.tissue()).unwrap().kind;
        match tissue_type {
            TissueType::Meristem => {
                let auxin = bio.auxin_level();
                let cytokinin = bio.cytokinin_level();
                if auxin > 0.8 && cytokinin < 0.2 {
                    differentiate_event_writer.write(CellDifferentiateEvent { cell: entity });
                }
            }
            TissueType::Parenchyma => {
                // do not differentiate
            }
        }
    }
}

pub fn post_update() {}
