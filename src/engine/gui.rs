use bevy::ecs::system::{Query, ResMut};
use bevy_egui::{
    egui::{self, Color32, RichText, ScrollArea},
    EguiContexts,
};

use crate::{
    engine::state::{PlantState, RunningState},
    model::{
        cell::Cell,
        organ::OrganType,
        tissue::{Tissue, TissueType},
    },
    shared::cell::CellInformation,
};

use super::{
    selection::Selected,
    state::{self, ApplicationState},
};

pub fn show_gui(
    mut contexts: EguiContexts,
    tissue_query: Query<(&Tissue, &Selected)>,
    cell_query: Query<(&CellInformation<f32>, &Cell, &Selected)>,
    mut app_state: ResMut<ApplicationState>,
    mut plant_state: ResMut<PlantState>,
) {
    show_application_state(&mut contexts, &mut app_state);
    show_plant_config(&mut contexts, &mut plant_state);
    match &*app_state {
        ApplicationState::Running(RunningState { level, .. }) => {
            match &level {
                state::Level::Cells => {
                    show_cells(contexts, cell_query);
                }
                state::Level::Tissues => {
                    show_tissues(contexts, tissue_query);
                }
            };
        }
    };
}

pub fn show_application_state(contexts: &mut EguiContexts, state: &mut ResMut<ApplicationState>) {
    match &mut **state {
        ApplicationState::Running(running_state) => {
            egui::Window::new("Editor").show(contexts.ctx_mut(), |ui| {
                ui.vertical(|ui| {
                    ui.label("Time: ");
                    ui.add(egui::Slider::new(&mut running_state.speed, 0.1..=120.));
                });
            });
        }
    }
}

pub fn show_plant_config(contexts: &mut EguiContexts, state: &mut ResMut<PlantState>) {
    egui::Window::new("Plant config").show(contexts.ctx_mut(), |ui| {
        let organs = vec![OrganType::Stem];
        let tissues = vec![TissueType::Meristem, TissueType::Parenchyma];
        for organ in organs.iter() {
            for tissue in tissues.iter() {
                if let Some(tissue_config) = state.tissue_config_mut(organ, tissue) {
                    ui.heading(tissue.to_string());
                    ui.vertical(|ui| {
                        ui.label("Maximum cell volume: ");
                        ui.add(egui::Slider::new(
                            &mut tissue_config.max_cell_volume,
                            1.0..=50.,
                        ));
                    });
                    ui.vertical(|ui| {
                        ui.label("Auxin production rate: ");
                        ui.add(egui::Slider::new(
                            &mut tissue_config.auxin_production_rate,
                            0.0..=0.001,
                        ));
                    });
                    ui.vertical(|ui| {
                        ui.label("Active hormone transport: ");
                        ui.add(egui::Slider::new(
                            &mut tissue_config.active_transport_factor,
                            0.0..=0.01,
                        ));
                    });
                    ui.vertical(|ui| {
                        ui.label("Diffusion rate: ");
                        ui.add(egui::Slider::new(
                            &mut tissue_config.diffusion_factor,
                            0.0..=0.001,
                        ));
                    });
                    if let Some(growing_tissue_config) = &mut tissue_config.growing_config {
                        ui.vertical(|ui| {
                            ui.label("Growing direction:");
                            ui.horizontal(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label("x : ");
                                    ui.add(egui::Slider::new(
                                        &mut growing_tissue_config.growing_direction.x,
                                        0.0..=6.,
                                    ));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("y : ");
                                    ui.add(egui::Slider::new(
                                        &mut growing_tissue_config.growing_direction.y,
                                        0.0..=6.,
                                    ));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("z : ");
                                    ui.add(egui::Slider::new(
                                        &mut growing_tissue_config.growing_direction.z,
                                        0.0..=6.,
                                    ));
                                });
                            });
                        });
                    }
                }
            }
        }
    });
}

pub fn show_tissues(mut contexts: EguiContexts, tissue_query: Query<(&Tissue, &Selected)>) {
    egui::Window::new("Tissues").show(contexts.ctx_mut(), |ui| {
        ScrollArea::vertical().max_height(500.).show(ui, |ui| {
            for (tissue, selected) in tissue_query.iter() {
                let tissue_string = format!("{} ({})", tissue.kind, tissue.cell_refs.len());
                let mut tissue_text = RichText::new(tissue_string);
                if selected.0 {
                    tissue_text = tissue_text.color(Color32::YELLOW);
                }
                ui.label(tissue_text);
            }
        });
    });
}

pub fn show_cells(
    mut contexts: EguiContexts,
    cell_query: Query<(&CellInformation<f32>, &Cell, &Selected)>,
) {
    egui::Window::new("Cells").show(contexts.ctx_mut(), |ui| {
        ScrollArea::vertical().max_height(500.).show(ui, |ui|{
            ui.vertical(|ui| {
                for (cell, bio, selected) in cell_query.iter() {
                    let cell_string = format!(
                        "Position: ({:.2}, {:.2}, {:.2}), Radius: {:.2}, Auxin: {:.2}, Cytokinin: {:.2}",
                        cell.position.x,
                        cell.position.y,
                        cell.position.z,
                        cell.radius,
                        bio.auxin_level(),
                        bio.cytokinin_level()
                    );
                    let mut text = RichText::new(cell_string);
                    if selected.0 {
                        text = text.color(Color32::YELLOW);
                    }
                    ui.label(text);
                }
            });
        });
    });
}
