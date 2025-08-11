use bevy::ecs::system::{Query, ResMut};
use bevy_egui::{
    egui::{self, Align2, Color32, RichText, ScrollArea},
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
            egui::Window::new("Editor")
                .anchor(Align2::LEFT_TOP, [5., 5.])
                .show(contexts.ctx_mut(), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Speed: ");
                        ui.add(egui::Slider::new(&mut running_state.speed, 0.1..=1000.));
                    });
                });
        }
    }
}

pub fn show_plant_config(contexts: &mut EguiContexts, state: &mut ResMut<PlantState>) {
    egui::Window::new("Plant config (".to_owned() + state.name().as_str() + ")")
        .anchor(Align2::LEFT_BOTTOM, [5., -5.])
        .default_open(false)
        .show(contexts.ctx_mut(), |ui| {
            let organs = vec![OrganType::Stem];
            let tissues = vec![TissueType::Meristem, TissueType::Parenchyma];
            let mut slider_value_changed = false;
            for organ in organs.iter() {
                for tissue in tissues.iter() {
                    if let Some(tissue_config) = state.tissue_config_mut(organ, tissue) {
                        ui.heading(tissue.to_string());
                        ui.horizontal(|ui| {
                            ui.label("Maximum cell volume: ");
                            let response = ui.add(egui::Slider::new(
                                &mut tissue_config.max_cell_volume,
                                1.0..=50.,
                            ));
                            if response.changed() {
                                slider_value_changed = true;
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Auxin:");
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Production rate: ");
                                    let response = ui.add(egui::Slider::new(
                                        &mut tissue_config.auxin_production_rate,
                                        0.0..=0.001,
                                    ));
                                    if response.changed() {
                                        slider_value_changed = true;
                                    }
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Inhibition start: ");
                                    let response = ui.add(egui::Slider::new(
                                        &mut tissue_config.auxin_inhibition_start,
                                        0.0..=1.5,
                                    ));
                                    if response.changed() {
                                        slider_value_changed = true;
                                    }
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Inhibition slope: ");
                                    let response = ui.add(egui::Slider::new(
                                        &mut tissue_config.auxin_inhibition_slope,
                                        0.0..=1.0,
                                    ));
                                    if response.changed() {
                                        slider_value_changed = true;
                                    }
                                });
                            });
                        });
                        ui.horizontal(|ui| {
                            ui.label("Diffusion rate: ");
                            let response = ui.add(egui::Slider::new(
                                &mut tissue_config.diffusion_factor,
                                0.0..=0.001,
                            ));
                            if response.changed() {
                                slider_value_changed = true;
                            }
                        });
                        if let Some(growing_tissue_config) = &mut tissue_config.growing_config {
                            ui.horizontal(|ui| {
                                ui.label("Active hormone transport rate: ");
                                let response = ui.add(egui::Slider::new(
                                    &mut growing_tissue_config.active_hormone_transport_rate,
                                    0.0..=0.001,
                                ));
                                if response.changed() {
                                    slider_value_changed = true;
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("Division:");
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label("Minimum auxin level: ");
                                        let response = ui.add(egui::Slider::new(
                                            &mut growing_tissue_config.divide_min_auxin,
                                            0.0..=2.0,
                                        ));
                                        if response.changed() {
                                            slider_value_changed = true;
                                        }
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Minimum volume percent: ");
                                        let response = ui.add(egui::Slider::new(
                                            &mut growing_tissue_config.divide_min_volume_percent,
                                            0.0..=100.0,
                                        ));
                                        if response.changed() {
                                            slider_value_changed = true;
                                        }
                                    });
                                });
                            });
                            ui.horizontal(|ui| {
                                ui.label("Growing direction:");
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label("x : ");
                                        let response = ui.add(egui::Slider::new(
                                            &mut growing_tissue_config.growing_direction.x,
                                            -1.0..=1.0,
                                        ));
                                        if response.changed() {
                                            slider_value_changed = true;
                                        }
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("y : ");
                                        let response = ui.add(egui::Slider::new(
                                            &mut growing_tissue_config.growing_direction.y,
                                            -1.0..=1.0,
                                        ));
                                        if response.changed() {
                                            slider_value_changed = true;
                                        }
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("z : ");
                                        let response = ui.add(egui::Slider::new(
                                            &mut growing_tissue_config.growing_direction.z,
                                            -1.0..=1.0,
                                        ));
                                        if response.changed() {
                                            slider_value_changed = true;
                                        }
                                    });
                                });
                            });
                        }
                    }
                }
                if slider_value_changed {
                    state.set_changed_from_ui(true);
                }
            }
        });
}

pub fn show_tissues(mut contexts: EguiContexts, tissue_query: Query<(&Tissue, &Selected)>) {
    egui::Window::new("Tissues")
        .anchor(Align2::RIGHT_TOP, [-5., 5.])
        .show(contexts.ctx_mut(), |ui| {
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
    egui::Window::new("Cells")
        .anchor(Align2::RIGHT_TOP, [-5.,5.])
        .show(contexts.ctx_mut(), |ui| {
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
