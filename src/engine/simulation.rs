use std::sync::Arc;

use bevy::prelude::*;

use crate::{
    engine::{
        cell_events::{CellDifferentiateEvent, CellDivideEvent},
        state::{ApplicationState, RunningState, SimulationTime},
    },
    model::{
        cell::Cell,
        hormone::{HormoneFlowEvent, Phytohormones},
        tissue::{Tissue, TissueType},
    },
    shared::{
        cell::{overlap, CellInformation},
        math::point_to_bevy_vec3,
        overlapping_cells::OverlappingCells,
    },
};

pub fn pre_update(mut commands: Commands, query: Query<(Entity, &CellInformation<f32>, &Cell)>) {
    let cells: Vec<(Entity, &CellInformation<f32>, &Cell)> = query.iter().collect();

    for (entity, info, _) in query.iter() {
        let neighbors: Vec<(Entity, CellInformation<f32>, Phytohormones)> = cells
            .iter()
            .filter(|(other_entity, other_info, _)| {
                *other_entity != entity && overlap(info, other_info)
            })
            .map(|(other_entity, other_info, other_cell)| {
                (
                    *other_entity,
                    (*other_info).clone(),
                    other_cell.hormones().clone(),
                )
            })
            .collect();

        // potentially empty neighbors but that's good
        commands.entity(entity).insert(OverlappingCells(neighbors));
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
    mut flow_events: EventWriter<HormoneFlowEvent>,
) {
    // update cell volume and hormones
    for (_, mut transform, mut cell, mut info, overlapping_cells) in &mut cell_query {
        let new_volume = cell.update_size(sim_time.elapsed);
        cell.simulate_hormones(
            sim_time.delta_secs,
            &info,
            overlapping_cells,
            &mut flow_events,
        );
        info.update(overlapping_cells, new_volume);
        transform.translation = point_to_bevy_vec3(&info.position);
        transform.scale.x = info.radius;
        transform.scale.y = info.radius;
        transform.scale.z = info.radius;
    }
    // divide cells if they reach a threshold
    for (entity, _, cell, _, _) in cell_query.iter_mut() {
        if let Ok(tissue) = tissue_query.get(cell.tissue()) {
            match tissue.kind {
                TissueType::Meristem => {
                    if cell.auxin_level() > 0.8 {
                        divide_event_writer.write(CellDivideEvent { parent: entity });
                    }
                }
                TissueType::Parenchyma => {
                    // do not divide
                }
            }
        } else {
            println!("Cannot find tissue with id {:?}", cell.tissue());
        };
    }
    // handle differentiation
    for (entity, _, cell, _, _) in cell_query.iter() {
        let tissue_type = &tissue_query.get(cell.tissue()).unwrap().kind;
        match tissue_type {
            TissueType::Meristem => {
                if cell.auxin_level() < 0.2 {
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
