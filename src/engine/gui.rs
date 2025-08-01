use bevy::ecs::system::{Query, Res};
use bevy_egui::{
    egui::{self, Color32, RichText},
    EguiContexts,
};

use crate::{
    model::{cell::BiologicalCell, tissue::Tissue},
    shared::cell::CellInformation,
};

use super::{
    selection::Selected,
    state::{self, ApplicationState},
};

pub fn show_tissues_or_cells(
    contexts: EguiContexts,
    tissue_query: Query<(&Tissue, &Selected)>,
    cell_query: Query<(&CellInformation<f32>, &BiologicalCell, &Selected)>,
    state: Res<ApplicationState>,
) {
    match &*state {
        ApplicationState::Running(level) => match &level {
            state::Level::Cells => {
                show_cells(contexts, cell_query);
            }
            state::Level::Tissues => {
                show_tissues(contexts, tissue_query);
            }
        },
    };
}

pub fn show_tissues(mut contexts: EguiContexts, tissue_query: Query<(&Tissue, &Selected)>) {
    egui::Window::new("Tissues").show(contexts.ctx_mut(), |ui| {
        for (tissue, selected) in tissue_query.iter() {
            let tissue_string = format!("{} ({})", tissue.tissue_type, tissue.cell_refs.len());
            let mut tissue_text = RichText::new(tissue_string);
            if selected.0 {
                tissue_text = tissue_text.color(Color32::YELLOW);
            }
            ui.label(tissue_text);
        }
    });
}

pub fn show_cells(
    mut contexts: EguiContexts,
    cell_query: Query<(&CellInformation<f32>, &BiologicalCell, &Selected)>,
) {
    egui::Window::new("Cells").show(contexts.ctx_mut(), |ui| {
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
}
