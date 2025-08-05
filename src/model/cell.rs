use bevy::ecs::{component::Component, entity::Entity};
use std::f32::consts::E;

use crate::model::{hormone::Phytohormones, tissue::TissueConfig};

#[derive(Clone, Copy, Debug)]
pub struct GrowthFactors {
    growth_factor: f32,
    start_value: f32,
    size_threshold: f32,
}

#[derive(Debug, Component)]
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

    pub fn update_hormones(&mut self, sim_delta_secs: f32) {
        let random_factor = rand::random::<f32>() * self.tissue_config.auxin_production_rate;
        self.hormones.auxin_level +=
            (self.tissue_config.auxin_production_rate + random_factor) * sim_delta_secs;
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
