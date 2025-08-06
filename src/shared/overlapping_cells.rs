use bevy::ecs::{component::Component, entity::Entity};
use cgmath::BaseFloat;

use crate::model::hormone::Phytohormones;

use super::cell::CellInformation;

#[derive(Component)]
pub struct OverlappingCells<T: BaseFloat>(pub Vec<(Entity, CellInformation<T>, Phytohormones)>);
