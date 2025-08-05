use bevy::ecs::system::{Query, ResMut};
use bevy_egui::{
    egui::{self, Color32, RichText, ScrollArea},
    EguiContexts,
};

use crate::{
    engine::state::RunningState,
    model::{cell::Cell, tissue::Tissue},
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
    mut state: ResMut<ApplicationState>,
) {
    show_editor(&mut contexts, &mut state);
    match &*state {
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

pub fn show_editor(contexts: &mut EguiContexts, state: &mut ResMut<ApplicationState>) {
    match &mut **state {
        ApplicationState::Running(running_state) => {
            egui::Window::new("Editor").show(contexts.ctx_mut(), |ui| {
                ui.vertical(|ui| {
                    ui.label("Time: ");
                    ui.add(egui::Slider::new(&mut running_state.speed, 0.1..=1000.));
                });
            });
        }
    }
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
