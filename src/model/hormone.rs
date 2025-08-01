#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Phytohormones {
    /// The amount of auxins, usually between 1 and 100 µg / kg plant material.
    ///
    /// Auxins are elongation hormones, mostly promoting growth
    pub auxin_level: f32,
    /// The amount of cytokinins
    pub cytokinin_level: f32,
}

impl Phytohormones {
    pub fn new() -> Self {
        Phytohormones {
            auxin_level: 0.0,
            cytokinin_level: 0.0,
        }
    }
}
