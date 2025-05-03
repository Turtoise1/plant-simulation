use bevy::ecs::system::{Query, Res};
use bevy_egui::{
    egui::{self, Color32, RichText, Vec2},
    EguiContexts,
};

use crate::{model::tissue::Tissue, shared::cell::CellInformation};

use super::{
    selection::Selected,
    state::{self, ApplicationState},
};

pub fn show_tissues_or_cells(
    contexts: EguiContexts,
    tissue_query: Query<(&Tissue, &Selected)>,
    cell_query: Query<(&CellInformation<f32>, &Selected)>,
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
            if selected.0 {
                ui.label(RichText::new(tissue.tissue_type.to_string()).color(Color32::YELLOW));
            } else {
                ui.label(tissue.tissue_type.to_string());
            }
        }
    });
}

pub fn show_cells(
    mut contexts: EguiContexts,
    cell_query: Query<(&CellInformation<f32>, &Selected)>,
) {
    egui::Window::new("Cells").show(contexts.ctx_mut(), |ui| {
        for (cell, selected) in cell_query.iter() {
            let cell_string = format!(
                "Position: ({:.2}, {:.2}, {:.2}), Radius: {:.2}",
                cell.position.x, cell.position.y, cell.position.z, cell.radius
            );
            if selected.0 {
                ui.label(RichText::new(cell_string).color(Color32::YELLOW));
            } else {
                ui.label(cell_string);
            }
        }
    });
}
