use bevy::{ecs::component::Component, math::Vec3};

#[derive(Component, Debug)]
pub struct Tissue {
    pub tissue_type: TissueType,
}

#[derive(Debug)]
pub struct GrowingTissue {
    pub growing_direction: Vec3,
}

#[derive(Debug)]
pub enum TissueType {
    /// A tissue of regularly dividing cells.
    Meristem(GrowingTissue),
    /// A tissue of far differentiated cells "filling up" many parts of a plant.
    Parenchyma,
}

impl Tissue {
    pub fn new(tissue_type: TissueType) -> Self {
        Tissue { tissue_type }
    }
}

impl GrowingTissue {
    pub fn new(growing_direction: Vec3) -> Self {
        Self { growing_direction }
    }
}
