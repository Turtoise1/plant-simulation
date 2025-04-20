use bevy::ecs::component::Component;
use cgmath::BaseFloat;

use super::cell::CellInformation;

#[derive(Component)]
pub struct OverlappingCells<T: BaseFloat>(pub Vec<CellInformation<T>>);
