use bevy::ecs::{component::Component, entity::Entity, event::EventWriter};
use std::f32::consts::E;

use crate::{
    model::{
        hormone::{HormoneFlowEvent, Phytohormones},
        tissue::TissueConfig,
    },
    shared::{cell::CellInformation, math, overlapping_cells::OverlappingCells},
};

#[derive(Clone, Copy, Debug)]
pub struct GrowthFactors {
    growth_factor: f32,
    start_value: f32,
    size_threshold: f32,
}

#[derive(Debug, Component, Clone)]
pub struct Cell {
    birth_time: f32,
    growth_factors: GrowthFactors,
    hormones: Phytohormones,
    /// reference to the tissue entity this cell belongs to
    tissue: Entity,
    tissue_config: TissueConfig,
}

impl Cell {
    pub fn new(
        volume: f32,
        tissue: Entity,
        sim_time: f32,
        hormones: Phytohormones,
        tissue_config: TissueConfig,
    ) -> Self {
        let cell = Cell {
            birth_time: sim_time,
            growth_factors: GrowthFactors {
                growth_factor: 0.0003,
                start_value: volume,
                size_threshold: tissue_config.max_cell_volume,
            },
            hormones,
            tissue,
            tissue_config,
        };
        cell
    }

    pub fn simulate_hormones(
        &mut self,
        sim_delta_secs: f32,
        info: &CellInformation<f32>,
        overlapping_cells: &OverlappingCells<f32>,
        flow_events: &mut EventWriter<HormoneFlowEvent>,
    ) {
        // auxin production
        let growing_sum = self.tissue_config.auxin_production_rate * sim_delta_secs;
        self.hormones.auxin_level += growing_sum * info.radius * sim_delta_secs;
        // auxin diffusion
        for (other_entity, _, other_hormones) in overlapping_cells.0.iter() {
            let gradient = self.auxin_level() - other_hormones.auxin_level;
            let flux = self.tissue_config.diffusion_factor * gradient * sim_delta_secs; // TODO more flux the more they overlap
            self.hormones.auxin_level -= flux;
            flow_events.write(HormoneFlowEvent {
                target_cell: *other_entity,
                amount: Phytohormones {
                    auxin_level: flux,
                    cytokinin_level: 0.,
                },
            });
        }
        // active auxin transport
        if let Some(growing_config) = &self.tissue_config.growing_config {
            if let Some((_, central_cell)) = &growing_config.central_cell {
                let target_position = central_cell.position
                    + math::bevy_vec3_to_vector3(&growing_config.growing_direction);
                let target_direction =
                    math::vector3_to_bevy_vec3(&math::direction(&info.position, &target_position));
                for (other_entity, other_info, other_hormones) in overlapping_cells.0.iter() {
                    let other_cell_dir = math::vector3_to_bevy_vec3(&math::direction(
                        &info.position,
                        &other_info.position,
                    ));

                    let deg_treshold = 30.;
                    let angle = target_direction.angle_between(other_cell_dir).to_degrees();

                    if self.auxin_level() > 0.0 && angle < deg_treshold {
                        let distance = math::distance(&info.position, &target_position);
                        let flux =
                            self.tissue_config.active_transport_factor * sim_delta_secs * distance; // TODO more flux the more they overlap
                        self.hormones.auxin_level -= flux;
                        flow_events.write(HormoneFlowEvent {
                            target_cell: *other_entity,
                            amount: Phytohormones {
                                auxin_level: flux,
                                cytokinin_level: 0.,
                            },
                        });
                    }
                }
            };
        }
    }

    pub fn add_to_hormone_level(&mut self, delta: Phytohormones) {
        self.hormones.auxin_level += delta.auxin_level;
        self.hormones.cytokinin_level += delta.cytokinin_level;
    }

    pub fn update_size(&mut self, sim_time: f32) -> f32 {
        let volume = logistic_growth(self.growth_factors)(sim_time - self.birth_time);
        volume
    }

    pub fn on_divide(&mut self, new_volume: f32, sim_time: f32) {
        self.birth_time = sim_time;
        self.growth_factors.start_value = new_volume;
        self.hormones.auxin_level /= 2.;
    }

    pub fn tissue(&self) -> Entity {
        self.tissue
    }

    pub fn update_tissue(&mut self, new_tissue: Entity) {
        self.tissue = new_tissue;
    }

    pub fn hormones(&self) -> &Phytohormones {
        &self.hormones
    }

    pub fn auxin_level(&self) -> f32 {
        self.hormones.auxin_level
    }

    pub fn cytokinin_level(&self) -> f32 {
        self.hormones.cytokinin_level
    }

    pub fn update_tissue_config(&mut self, tissue_config: TissueConfig) {
        self.growth_factors.size_threshold = tissue_config.max_cell_volume;
        self.tissue_config = tissue_config;
    }
}

fn logistic_growth(growth_factors: GrowthFactors) -> impl Fn(f32) -> f32 {
    // f'(t)=k*f(t)*(G-f(t))
    // => f(t)=1/(1+e^(-k*G*t)*(G/f(0)-1))
    return move |t: f32| {
        growth_factors.size_threshold
            / (1.
                + f32::powf(
                    E,
                    -growth_factors.growth_factor * growth_factors.size_threshold * t as f32,
                ) * (growth_factors.size_threshold / growth_factors.start_value - 1.))
    };
}
